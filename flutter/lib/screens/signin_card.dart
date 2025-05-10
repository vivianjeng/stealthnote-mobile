import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:markdown_editor_plus/markdown_editor_plus.dart';
import '../firebase_options.dart';
import '../services/auth_service.dart';
import '../services/create_membershipt.dart';
import '../services/create_message.dart';
import '../services/fetch_googleJWTpubKey.dart';
import '../services/generate_ephemeral_key.dart';
import '../services/google_jwt_prover.dart';
import '../services/jwt_prover.dart';

class SignInCard extends StatefulWidget {
  final VoidCallback onPostSuccess;
  final bool isInternal;

  const SignInCard(
      {Key? key, required this.onPostSuccess, required this.isInternal})
      : super(key: key);

  @override
  _SignInCardState createState() => _SignInCardState();
}

class _SignInCardState extends State<SignInCard> {
  final AuthService _authService = AuthService();
  final moproFlutterPlugin = MoproFlutter();
  bool _isLoading = false;
  final TextEditingController _textController = TextEditingController();

  @override
  void initState() {
    super.initState();
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  // Google Sign-In function using AuthService
  // Update the _signInWithGoogle method in _SignInPageState class to navigate to HomePage
  // Then update the _signInWithGoogle method:
  Future<void> _signInWithGoogle() async {
    if (mounted) {
      setState(() {
        _isLoading = true;
      });
    }

    try {
      final ephemeralKey = await getEphemeralKey();

      // Decode the JSON string
      Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
      final ephemeralPubkey = ephemeralKeyObj['public_key'];
      final ephemeralExpiry = ephemeralKeyObj['expiry'];
      final ephemeralPubkeyHash = ephemeralKeyObj['pubkey_hash'];

      final String? idToken = await _authService.signInManually(
        ephemeralPubkeyHash,
      );
      final credential = GoogleAuthProvider.credential(idToken: idToken);
      final UserCredential? userCredential =
          await _authService.signInWithGoogle(credential);

      if (userCredential != null && userCredential.user != null) {
        // Navigate to BottomNavBar instead of directly to HomePage
        if (mounted) {
          // Store user information (you may want to save this in a shared preferences or state management)
          final String userEmail = userCredential.user!.email!;
          final String? displayName = userCredential.user!.displayName;

          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Signed in as: $userEmail'),
              backgroundColor: Colors.green,
            ),
          );
        }
      } else {
        // User cancelled the sign-in flow
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Sign-in canceled'),
              backgroundColor: Colors.orange,
            ),
          );
        }
      }

      // generate jwt proof
      // final idToken = googleAuth.idToken;
      final header = parseJwtHeader(idToken);
      final payload = parseJwtPayload(idToken);
      final googlePublicKey = await fetchGooglePublicKey(header['kid']);

      final proof = await generateJwtProof(
        jsonEncode(googlePublicKey),
        idToken,
        sliceEmail(payload['email']),
      );

      // create membership
      final proofArgs = {"keyId": header['kid'], "jwtCircuitVersion": "0.3.1"};
      await createMembership(
        ephemeralPubkey,
        ephemeralExpiry,
        sliceEmail(payload['email']),
        "google-oauth",
        proof!,
        proofArgs,
      );
    } catch (e) {
      // Handle any errors that occur during the sign-in process
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Sign-in error: ${e.toString()}'),
            backgroundColor: Colors.red,
          ),
        );
      }
      print('Error signing in with Google: $e');
    } finally {
      // Reset loading state
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  String sliceEmail(dynamic email) {
    return email.substring(email.indexOf('@') + 1);
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            MarkdownAutoPreview(
              controller: _textController,
              emojiConvert: true,
              decoration: InputDecoration(
                hintText: "What's happening at your company?",
                border: InputBorder.none,
                contentPadding: EdgeInsets.zero,
              ),
              maxLines: 5,
              minLines: 3,
            ),
            const SizedBox(height: 12),
            StreamBuilder<User?>(
              stream: _authService.authStateChanges,
              builder: (context, snapshot) {
                if (snapshot.hasData && snapshot.data != null) {
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          Expanded(
                            child: Text(
                              "Posting as \"Someone from ${sliceEmail(snapshot.data!.email)}\"",
                              style: TextStyle(fontSize: 16),
                            ),
                          ),
                          IconButton(
                            icon: Icon(Icons.refresh),
                            onPressed: () async {
                              await _authService.signOut();
                              await _signInWithGoogle();
                            },
                          ),
                          IconButton(
                            icon: Icon(Icons.close),
                            onPressed: () async {
                              await _authService.signOut();
                            },
                          ),
                          const SizedBox(width: 8),
                          ElevatedButton(
                            onPressed: () {
                              // Access the text when the post button is pressed
                              final text = _textController.text;
                              if (text.isNotEmpty) {
                                createMessage(
                                  text,
                                  sliceEmail(snapshot.data!.email),
                                  widget.isInternal, // internal
                                ).then((_) {
                                  _textController
                                      .clear(); // Clear the text field after posting
                                  widget
                                      .onPostSuccess(); // Call the callback after successful post
                                });
                              }
                            },
                            child: Text('Post'),
                            style: ElevatedButton.styleFrom(
                              backgroundColor: Color(0xFF3730A3),
                              foregroundColor: Colors.white,
                              padding: EdgeInsets.symmetric(
                                horizontal: 12,
                                vertical: 6,
                              ),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(4),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ],
                  );
                } else {
                  return Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      Expanded(
                        child: Text(
                          'Sign in with your Google work account to anonymously post as "Someone from your company".',
                          style: TextStyle(
                            fontStyle: FontStyle.italic,
                            fontSize: 14,
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Material(
                        color: Colors.transparent,
                        child: InkWell(
                          borderRadius: BorderRadius.circular(8),
                          onTap: _signInWithGoogle,
                          child: Container(
                            padding: EdgeInsets.all(6),
                            decoration: BoxDecoration(
                              color: Colors.white,
                              borderRadius: BorderRadius.circular(8),
                              boxShadow: [
                                BoxShadow(
                                  color: Colors.black12,
                                  blurRadius: 4,
                                  offset: Offset(0, 2),
                                ),
                              ],
                              border: Border.all(color: Colors.grey.shade300),
                            ),
                            child: Image.asset(
                              'assets/google.png',
                              width: 36,
                              height: 36,
                            ),
                          ),
                        ),
                      ),
                    ],
                  );
                }
                return const SizedBox.shrink();
              },
            ),
          ],
        ),
      ),
    );
  }
}

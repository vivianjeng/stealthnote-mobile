import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:markdown_editor_plus/markdown_editor_plus.dart';
import 'package:image_picker/image_picker.dart';
import '../firebase_options.dart';
import '../services/auth_service.dart';
import '../services/create_membershipt.dart';
import '../services/create_message.dart';
import '../services/fetch_googleJWTpubKey.dart';
import '../services/generate_ephemeral_key.dart';
import '../services/google_jwt_prover.dart';
import '../services/jwt_prover.dart';
import '../services/upload_image.dart';

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
  bool _isUploadingImage = false;
  final TextEditingController _textController = TextEditingController();
  XFile? _selectedImageFile;
  final ImagePicker _picker = ImagePicker();

  @override
  void initState() {
    super.initState();
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  Future<void> _pickImage() async {
    if (_isUploadingImage) return;
    final XFile? image = await _picker.pickImage(source: ImageSource.gallery);
    setState(() {
      _selectedImageFile = image;
    });
  }

  void _clearImage() {
    if (_isUploadingImage) return;
    setState(() {
      _selectedImageFile = null;
    });
  }

  Future<void> _initiateImageUpload(String anonGroupId) async {
    if (_selectedImageFile == null || _isUploadingImage) return;

    setState(() {
      _isUploadingImage = true;
    });

    final imagePath = _selectedImageFile!.path;

    try {
      File imageFile = File(imagePath);
      String? imageUrl = await uploadToImgur(imageFile);

      if (mounted && imageUrl != null) {
        String currentText = _textController.text;
        String imageMarkdown = '![image]($imageUrl)';
        
        if (currentText.isEmpty) {
          _textController.text = imageMarkdown;
        } else {
          String newText = currentText.endsWith('\n') ? currentText : currentText + '\n';
          _textController.text = newText + imageMarkdown;
        }
        
        _textController.selection = TextSelection.fromPosition(
          TextPosition(offset: _textController.text.length),
        );

        _selectedImageFile = null;
      } else if (mounted && imageUrl == null) {
         ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Image upload failed: Could not get image URL.')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Image upload failed: ${e.toString()}')),
        );
      }
      print('Error uploading image: $e');
    } finally {
      if (mounted) {
        setState(() {
          _isUploadingImage = false;
        });
      }
    }
  }

  Future<void> _signInWithGoogle() async {
    if (mounted) {
      setState(() {
        _isLoading = true;
      });
    }

    try {
      final ephemeralKey = await getEphemeralKey();

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
        if (mounted) {
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
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Sign-in canceled'),
              backgroundColor: Colors.orange,
            ),
          );
        }
      }

      final header = parseJwtHeader(idToken);
      final payload = parseJwtPayload(idToken);
      final googlePublicKey = await fetchGooglePublicKey(header['kid']);

      final proof = await generateJwtProof(
        jsonEncode(googlePublicKey),
        idToken,
        sliceEmail(payload['email']),
      );

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

  Widget _buildImageSelectionRow(User? currentUser) {
    return Row(
      children: [
        IconButton(
          icon: Icon(Icons.attach_file),
          onPressed: _isUploadingImage ? null : _pickImage,
          tooltip: 'Attach image',
        ),
        if (_selectedImageFile != null)
          Expanded(
            child: Row(
              children: [
                Icon(Icons.image_outlined, color: Colors.grey[700]),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    _selectedImageFile!.name,
                    style: TextStyle(fontSize: 12),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                SizedBox(width: 4),
                if (_isUploadingImage)
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 8.0),
                    child: SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    ),
                  )
                else if (currentUser != null)
                  TextButton(
                    onPressed: () => _initiateImageUpload(sliceEmail(currentUser.email!)),
                    child: Text("Upload Image"),
                  ),
                IconButton(
                  icon: Icon(Icons.close, size: 18),
                  onPressed: _isUploadingImage ? null : _clearImage,
                  tooltip: 'Remove image',
                ),
              ],
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: StreamBuilder<User?>(
          stream: _authService.authStateChanges,
          builder: (context, snapshot) {
            final User? currentUser = snapshot.data;
            return Column(
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
                _buildImageSelectionRow(currentUser),
                const SizedBox(height: 12),
                if (currentUser != null) ...[
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      Expanded(
                        child: Text(
                          "Posting as \"Someone from ${sliceEmail(currentUser.email)}\"",
                          style: TextStyle(fontSize: 16),
                        ),
                      ),
                      IconButton(
                        icon: Icon(Icons.refresh),
                        onPressed: _isLoading ? null : () async {
                          setState(() => _isLoading = true);
                          await _authService.signOut();
                          await _signInWithGoogle();
                        },
                      ),
                      IconButton(
                        icon: Icon(Icons.close),
                        onPressed: _isLoading ? null : () async {
                          await _authService.signOut();
                        },
                      ),
                      const SizedBox(width: 8),
                      ElevatedButton(
                        onPressed: _isLoading || _isUploadingImage ? null : () {
                          final text = _textController.text;
                          if (text.isNotEmpty || _selectedImageFile != null) {
                            setState(() => _isLoading = true);
                            createMessage(
                              text,
                              sliceEmail(currentUser.email!),
                              widget.isInternal,
                              _selectedImageFile?.path,
                            ).then((returnedMessageContent) {
                              if (mounted) {
                                _textController.text = returnedMessageContent;
                                if (_selectedImageFile != null) {
                                   _selectedImageFile = null;
                                }
                                widget.onPostSuccess();
                              }
                            }).catchError((e) {
                               if (mounted) {
                                ScaffoldMessenger.of(context).showSnackBar(
                                  SnackBar(content: Text('Failed to post: ${e.toString()}')));
                               }
                            }).whenComplete(() {
                              if (mounted) {
                                setState(() => _isLoading = false);
                              }
                            });
                          }
                        },
                        child: _isLoading && !_isUploadingImage
                            ? SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                            : Text('Post'),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: Color(0xFF3730A3),
                          foregroundColor: Colors.white,
                          padding: EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
                        ),
                      ),
                    ],
                  ),
                ] else ...[
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      Expanded(
                        child: Text(
                          'Sign in with your Google work account to anonymously post as "Someone from your company".',
                          style: TextStyle(fontStyle: FontStyle.italic, fontSize: 14),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Material(
                        color: Colors.transparent,
                        child: InkWell(
                          borderRadius: BorderRadius.circular(8),
                          onTap: _isLoading ? null : _signInWithGoogle,
                          child: _isLoading
                              ? Padding(
                                  padding: const EdgeInsets.all(6.0),
                                  child: SizedBox(width: 36, height: 36, child: CircularProgressIndicator()),
                                )
                              : Container(
                                  padding: EdgeInsets.all(6),
                                  decoration: BoxDecoration(
                                    color: Colors.white,
                                    borderRadius: BorderRadius.circular(8),
                                    boxShadow: [
                                      BoxShadow(color: Colors.black12, blurRadius: 4, offset: Offset(0, 2)),
                                    ],
                                    border: Border.all(color: Colors.grey.shade300),
                                  ),
                                  child: Image.asset('assets/google.png', width: 36, height: 36),
                                ),
                        ),
                      ),
                    ],
                  ),
                ],
              ],
            );
          },
        ),
      ),
    );
  }
}

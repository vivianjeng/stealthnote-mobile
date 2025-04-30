import 'package:flutter/material.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_auth/firebase_auth.dart';
import '../firebase_options.dart';
import '../services/auth_service.dart';
import '../services/jwt_prover.dart';

class SignInCard extends StatefulWidget {
  @override
  _SignInCardState createState() => _SignInCardState();
}

class _SignInCardState extends State<SignInCard> {
  final AuthService _authService = AuthService();
  bool _isLoading = false;

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
      // Use the AuthService to handle Google Sign-In
      final UserCredential? userCredential =
          await _authService.signInWithGoogle();

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
            TextField(
              decoration: InputDecoration(
                hintText: 'What\'s happening at your company?',
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
                              await _signInWithGoogle();
                              await _authService.signOut();
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
                            onPressed: () {},
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

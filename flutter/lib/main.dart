import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'screens/signin_card.dart';
import 'screens/message_card.dart';
import 'services/jwt_prover.dart';
import 'firebase_options.dart';
import 'services/auth_service.dart';

void main() async {
  // Initialize Flutter binding
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize Firebase - make sure you've added the necessary configuration files
  await Firebase.initializeApp(
    options: DefaultFirebaseOptions.currentPlatform,
  );

  runApp(StealthNoteApp());
}

class StealthNoteApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'StealthNote',
      theme: ThemeData(fontFamily: GoogleFonts.inconsolata().fontFamily),
      home: StealthHomePage(),
    );
  }
}

class StealthHomePage extends StatefulWidget {
  @override
  _StealthHomePageState createState() => _StealthHomePageState();
}


class _StealthHomePageState extends State<StealthHomePage> {

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


  final List<Message> messages = [
    Message(
      org: 'coinspect.com',
      time: DateTime.now().subtract(Duration(days: 1)),
      body:
          'Fantastic work on StealthNote! One gap: the Trust assumptions / Potential attacks...',
    ),
    Message(
      org: 'ethereum.org',
      time: DateTime.now().subtract(Duration(days: 4)),
      body:
          '@chainbound.io person – probably related to the stated "open problem"...',
    ),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('StealthNote', style: GoogleFonts.inconsolata()),
        backgroundColor: Colors.white,
        foregroundColor: Colors.black,
        elevation: 0,
        leading: Icon(Icons.menu),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          SignInCard(),
          ElevatedButton.icon(
            onPressed: _signInWithGoogle,
            icon: Image.network(
              'https://www.google.com/favicon.ico',
              height: 24.0,
              errorBuilder:
                  (context, error, stackTrace) =>
                      const Icon(Icons.g_mobiledata),
            ),
            label: const Text(
              'Sign in with Google',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
            ),
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.white,
              foregroundColor: Colors.black87,
              padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
                side: BorderSide(color: Colors.grey.shade300),
              ),
            ),
          ),
          const SizedBox(height: 16),
          // Sign out button (visible when signed in)
          StreamBuilder<User?>(
            stream: _authService.authStateChanges,
            builder: (context, snapshot) {
              if (snapshot.hasData && snapshot.data != null) {
                return Column(
                  children: [
                    Text(
                      'Signed in as: ${snapshot.data!.email}',
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    const SizedBox(height: 8),
                    ElevatedButton(
                      onPressed: () async {
                        await _authService.signOut();
                        if (mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Signed out successfully'),
                              backgroundColor: Colors.blue,
                            ),
                          );
                        }
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.red.shade400,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('Sign Out'),
                    ),
                  ],
                );
              }
              return const SizedBox.shrink();
            },
          ),
          const SizedBox(height: 16),
          ...messages.map((msg) => MessageCard(msg)).toList(),
        ],
      ),
    );
  }
}

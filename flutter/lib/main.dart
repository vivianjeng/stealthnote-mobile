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
          const SizedBox(height: 16),
          ...messages.map((msg) => MessageCard(msg)).toList(),
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:google_fonts/google_fonts.dart';
import 'screens/signin_card.dart';
import 'screens/message_card.dart';
import 'services/fetch_messages.dart';
import 'services/jwt_prover.dart';
import 'firebase_options.dart';
import 'services/auth_service.dart';

void main() async {
  // Initialize Flutter binding
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize Firebase - make sure you've added the necessary configuration files
  await Firebase.initializeApp(options: DefaultFirebaseOptions.currentPlatform);

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
  List<Message> messages = [];
  final ScrollController _scrollController = ScrollController();
  bool isInternal = false;
  User? _user;
  int _messageKey = 0;
  bool _isLoadingMore = false;
  bool _hasMoreMessages = true;
  DateTime? _oldestMessageTime;

  @override
  void initState() {
    super.initState();
    _loadMessages();
    // Listen to auth state changes
    FirebaseAuth.instance.authStateChanges().listen((User? user) {
      setState(() {
        _user = user;
      });
    });
    // Add scroll listener
    _scrollController.addListener(_onScroll);
  }

  void _onScroll() {
    if (_scrollController.position.pixels >= _scrollController.position.maxScrollExtent - 200 && 
        !_isLoadingMore && 
        _hasMoreMessages) {
      _loadMoreMessages();
    }
  }

  String sliceEmail(dynamic email) {
    return email.substring(email.indexOf('@') + 1);
  }

  Future<void> _loadMessages() async {
    try {
      String? groupId = null;
      if (isInternal && _user != null) {
        groupId = sliceEmail(_user!.email);
      }
      final fetchedMessages = await fetchMessages(
          limit: 5, isInternal: isInternal, groupId: groupId);
      if (fetchedMessages != null && fetchedMessages.isNotEmpty) {
        List<Message> processedMessages = [];
        for (var message in fetchedMessages) {
          final msg = Message(
            id: message['id'],
            org: message['anonGroupId'],
            time: DateTime.parse(message['timestamp']),
            body: message['text'],
            likes: message['likes'],
            isLiked: 0,
            internal: message['internal'],
          );
          processedMessages.add(msg);
        }
        setState(() {
          messages = processedMessages;
          _messageKey++;
          _oldestMessageTime = processedMessages.last.time;
          _hasMoreMessages = fetchedMessages.length >= 5; // If we got less than limit, no more messages
        });
      }
    } catch (e) {
      print('Error loading messages: $e');
    }
  }

  Future<void> _loadMoreMessages() async {
    if (_isLoadingMore || !_hasMoreMessages || _oldestMessageTime == null) return;

    setState(() {
      _isLoadingMore = true;
    });

    try {
      String? groupId = null;
      if (isInternal && _user != null) {
        groupId = sliceEmail(_user!.email);
      }
      
      final fetchedMessages = await fetchMessages(
        limit: 5,
        isInternal: isInternal,
        groupId: groupId,
        beforeTimestamp: _oldestMessageTime!.millisecondsSinceEpoch,
      );

      if (fetchedMessages != null && fetchedMessages.isNotEmpty) {
        List<Message> newMessages = [];
        for (var message in fetchedMessages) {
          final msg = Message(
            id: message['id'],
            org: message['anonGroupId'],
            time: DateTime.parse(message['timestamp']),
            body: message['text'],
            likes: message['likes'],
            isLiked: 0,
            internal: message['internal'],
          );
          newMessages.add(msg);
        }

        setState(() {
          messages.addAll(newMessages);
          _oldestMessageTime = newMessages.last.time;
          _hasMoreMessages = fetchedMessages.length >= 5; // If we got less than limit, no more messages
        });
      } else {
        setState(() {
          _hasMoreMessages = false;
        });
      }
    } catch (e) {
      print('Error loading more messages: $e');
    } finally {
      setState(() {
        _isLoadingMore = false;
      });
    }
  }

  @override
  void dispose() {
    _scrollController.removeListener(_onScroll);
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      drawer: Drawer(
        child: ListView(
          padding: EdgeInsets.zero,
          children: [
            const DrawerHeader(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('StealthNote',
                      style:
                          TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
                ],
              ),
            ),
            ListTile(
              title: const Text('Home'),
              onTap: () {
                Navigator.pop(context);
                setState(() {
                  isInternal = false;
                  messages = [];
                  _oldestMessageTime = null;
                  _hasMoreMessages = true;
                });
                _loadMessages();
              },
            ),
            if (_user != null) ...[
              ListTile(
                title: Text('${sliceEmail(_user!.email)} Internal'),
                onTap: () {
                  Navigator.pop(context);
                  setState(() {
                    isInternal = true;
                    messages = [];
                    _oldestMessageTime = null;
                    _hasMoreMessages = true;
                  });
                  _loadMessages();
                },
              ),
            ],
          ],
        ),
      ),
      appBar: AppBar(
        title: Text('StealthNote', style: GoogleFonts.inconsolata()),
        backgroundColor: Colors.white,
        foregroundColor: Colors.black,
        elevation: 0,
      ),
      body: GestureDetector(
        onTap: () {
          FocusScope.of(context).unfocus();
        },
        child: RefreshIndicator(
          onRefresh: () async {
            setState(() {
              messages = [];
              _oldestMessageTime = null;
              _hasMoreMessages = true;
            });
            await _loadMessages();
          },
          child: ListView(
            controller: _scrollController,
            padding: const EdgeInsets.all(16.0),
            children: [
              SignInCard(
                isInternal: isInternal,
                onPostSuccess: () {
                  setState(() {
                    messages = [];
                    _oldestMessageTime = null;
                    _hasMoreMessages = true;
                  });
                  _loadMessages();
                },
              ),
              const SizedBox(height: 16),
              ...messages.map((msg) => MessageCard(msg, key: ValueKey('${msg.id}_$_messageKey'))).toList(),
              if (_isLoadingMore)
                const Center(
                  child: Padding(
                    padding: EdgeInsets.all(16.0),
                    child: CircularProgressIndicator(),
                  ),
                ),
              if (!_hasMoreMessages && messages.isNotEmpty)
                const Center(
                  child: Padding(
                    padding: EdgeInsets.all(16.0),
                    child: Text('No more messages'),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

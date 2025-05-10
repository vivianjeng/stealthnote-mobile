import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:timeago/timeago.dart' as timeago;
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../services/fetch_messages.dart';
import '../services/fetch_googleJWTpubKey.dart';
import '../services/toggle_like.dart';

class Message {
  final String id;
  final String org;
  final DateTime time;
  final String body;
  int likes;
  int isLiked;
  final bool internal;

  Message({
    required this.id,
    required this.org,
    required this.time,
    required this.body,
    required this.likes,
    required this.isLiked,
    required this.internal,
  });
}

class MessageCard extends StatefulWidget {
  final Message msg;
  const MessageCard(this.msg, {Key? key}) : super(key: key);

  @override
  _MessageCardState createState() => _MessageCardState();
}

class _MessageCardState extends State<MessageCard> {
  final _moproFlutterPlugin = MoproFlutter();

  VerifyJwtProofResult? _verificationResult;
  String? _srsPath;
  String? _status;
  String? _errorMessage;
  bool _isBusy = false;
  int? _verifyingTimeMillis;
  bool _isLoading = false;
  bool? _verified; // null = not verified yet

  @override
  void initState() {
    super.initState();
    // Copy assets needed for Jwt operations
    _copyAssets();
  }

  Future<void> _copyAssets() async {
    if (!mounted) return;
    
    setState(() {
      _status = 'Copying assets...';
      _errorMessage = null;
      _isBusy = true;
    });
    try {
      // Define asset paths relative to the 'assets' folder in pubspec.yaml
      const srsAssetPath = 'assets/jwt-srs.local';

      // Copy assets to the file system and store their paths
      final srsPath = await _moproFlutterPlugin.copyAssetToFileSystem(
        srsAssetPath,
      );

      if (!mounted) return;
      
      setState(() {
        _srsPath = srsPath;
        _status = 'Assets copied successfully. Ready.';
        _isBusy = false;
      });
    } catch (e) {
      if (!mounted) return;
      
      setState(() {
        _status = 'Error copying assets';
        _errorMessage = e.toString();
        _isBusy = false;
      });
      print("Error copying assets: $e");
    }
  }

  Uint8List toUint8List(List<dynamic> data) {
    return Uint8List.fromList(data.cast<int>());
  }

  // Function to call verifyJwtProof
  Future<void> _callVerifyJwtProof(String id, bool isInternal) async {
    setState(() {
      _isLoading = true;
      _verified = null;
    });
    if (_srsPath == null) {
      setState(() {
        _status = 'Proof not available or SRS path missing';
        _errorMessage = 'Generate a proof first or ensure SRS path is valid.';
      });
      return;
    }

    setState(() {
      _status = 'Verifying proof...';
      _verificationResult = null; // Clear previous verification result
      _errorMessage = null;
      _isBusy = true; // Start busy state
    });

    try {
      // Verify the proof
      final stopwatch = Stopwatch()..start();
      final message = await fetchMessage(id, isInternal);
      final googleJwtPubkeyModulus = await fetchGooglePublicKey(
        message['proofArgs']['keyId'],
      );
      final result = await _moproFlutterPlugin.verifyJwtProof(
        _srsPath!,
        Uint8List.fromList(message['proof'].cast<int>()),
        message['anonGroupId'],
        googleJwtPubkeyModulus?["n"],
        message['ephemeralPubkey'],
        message['ephemeralPubkeyExpiry'],
      );
      stopwatch.stop();

      setState(() {
        _verificationResult = result;
        _verifyingTimeMillis = stopwatch.elapsedMilliseconds;
        _status = result != null
            ? 'Verification finished.'
            : 'Verification failed (result is null)';
        _isBusy = false; // End busy state
        _isLoading = false;
        _verified = result?.isValid;
      });
    } catch (e) {
      setState(() {
        _status = 'Error verifying proof';
        _errorMessage = e.toString();
        _isBusy = false; // End busy state on error
        _isLoading = false;
        _verified = false;
      });
      print("Error verifying proof: $e");
    }
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Image.network(
                  "https://img.logo.dev/${widget.msg.org}?token=pk_SqdEexoxR3akcyJz7PneXg",
                  width: 24,
                  height: 24,
                ),
                const SizedBox(width: 8),
                Expanded(child: Text('Someone from ${widget.msg.org}')),
                Text(timeago.format(widget.msg.time)),
              ],
            ),
            const SizedBox(height: 12),
            MarkdownBody(data: widget.msg.body),
            const SizedBox(height: 8),
            Row(
              children: [
                IconButton(
                  onPressed: () async {
                    if (widget.msg.isLiked == 0) {
                      await toggleLike(widget.msg.id, true);
                      setState(() {
                        widget.msg.likes++;
                        widget.msg.isLiked = 1;
                      });
                    } else {
                      await toggleLike(widget.msg.id, false);
                      setState(() {
                        widget.msg.likes--;
                        widget.msg.isLiked = 0;
                      });
                    }
                  },
                  icon: Icon(widget.msg.isLiked == 1 ? Icons.thumb_up : Icons.thumb_up_alt_outlined, size: 16),
                ),
                const SizedBox(width: 4),
                Text(widget.msg.likes.toString()),
                const Spacer(),
                if (_verified != null)
                  Text(_verified! ? '✅ Verified' : '❌ Not verified')
                else
                  TextButton(
                    onPressed: _isLoading
                        ? null
                        : () => _callVerifyJwtProof(
                              widget.msg.id,
                              widget.msg.internal,
                            ),
                    child: _isLoading
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Verify'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

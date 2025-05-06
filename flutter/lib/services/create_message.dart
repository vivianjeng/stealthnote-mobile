import 'dart:convert';
import 'package:http/http.dart' as http;

import '../models/signed_message.dart';
import './generate_ephemeral_key.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

Future<void> createMessage(
  String content,
  String anonGroupId,
  bool internal,
) async {
  final moproFlutterPlugin = MoproFlutter();
  final ephemeralKey = await getEphemeralKey();

  // Decode the JSON string
  Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
  final ephemeralPubkey = ephemeralKeyObj['public_key'];
  final ephemeralExpiry = ephemeralKeyObj['expiry'];
  final ephemeralPrivateKey = ephemeralKeyObj['private_key'];
  print('content: $content');

  try {
    final signedMessage = await moproFlutterPlugin.signMessage(
      anonGroupId,
      content,
      internal,
      ephemeralPubkey,
      ephemeralPrivateKey,
      ephemeralExpiry,
    );

    // Send the signed message to the API
    final response = await http.post(
      Uri.parse('https://stealthnote.xyz/api/messages'),
      headers: {'Content-Type': 'application/json'},
      body: signedMessage,
    );

    if (response.statusCode != 201) {
      print('Error posting message: ${response.statusCode}');
      print('Response body: ${response.body}');
    }
  } catch (e) {
    print('Error creating message: $e');
  }
}

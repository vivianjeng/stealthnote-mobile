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
  Map<String, dynamic> ephemeral_key_obj = jsonDecode(ephemeralKey);
  final ephemeral_pubkey = ephemeral_key_obj['public_key'];
  final ephemeral_expiry = ephemeral_key_obj['expiry'];
  final ephemeral_private_key = ephemeral_key_obj['private_key'];
  print('content: $content');

  try {
    final signedMessage = await moproFlutterPlugin.signMessage(
      anonGroupId,
      content,
      internal,
      ephemeral_pubkey,
      ephemeral_private_key,
      ephemeral_expiry,
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

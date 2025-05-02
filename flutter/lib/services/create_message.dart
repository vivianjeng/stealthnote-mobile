import 'dart:convert';
import 'package:http/http.dart' as http;

import '../models/signed_message.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

Future<void> createMessage(
  String content,
  String anonGroupId,
  bool internal,
) async {
  final moproFlutterPlugin = MoproFlutter();
  final ephemeralPubkeyHash =
      "622618718926420486498127001071856504322492650656283936596477869965459887546";
  final expiry = "2025-05-07T09:07:57.379Z";
  final privateKey =
      "39919031573819484966641096195810516976016707561507350566056652693882791321787";
  final publicKey =
      "17302102366996071265028731047581517700208166805377449770193522591062772282670";
  final salt =
      "646645587996092179008704451306999156519169540151959619716525865713892520";
  print('content: $content');

  try {
    final signedMessage = await moproFlutterPlugin.signMessage(
      anonGroupId,
      content,
      internal,
      publicKey,
      privateKey,
      expiry,
    );

    // Send the signed message to the API
    final response = await http.post(
      Uri.parse('https://ac1f-125-229-173-139.ngrok-free.app/api/messages'),
      headers: {'Content-Type': 'application/json'},
      body: signedMessage,
    );

    if (response.statusCode != 200) {
      print('Error posting message: ${response.statusCode}');
      print('Response body: ${response.body}');
    }
  } catch (e) {
    print('Error creating message: $e');
  }
}

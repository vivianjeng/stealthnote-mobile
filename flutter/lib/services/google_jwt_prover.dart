import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:typed_data';
import '../models/signed_message.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

import 'generate_ephemeral_key.dart';

Future<Uint8List?> generateJwtProof(
  String jwt,
  String? idToken,
  String domain,
) async {
  if (idToken == null) {
    throw Exception('ID Token is null');
  }
  final moproFlutterPlugin = MoproFlutter();
  const srsAssetPath = 'assets/jwt-srs.local';
  final srsPath = await moproFlutterPlugin.copyAssetToFileSystem(srsAssetPath);
  final ephemeralKey = await getEphemeralKey();
  print('ephemeralKey: $ephemeralKey');

  // Decode the JSON string
  Map<String, dynamic> ephemeral_key_obj = jsonDecode(ephemeralKey);
  final ephemeral_pubkey = ephemeral_key_obj['public_key'];
  final ephemeral_salt = ephemeral_key_obj['salt'];
  final ephemeral_expiry = ephemeral_key_obj['expiry'];

  try {
    final proof = await moproFlutterPlugin.proveJwt(
      srsPath,
      ephemeral_pubkey,
      ephemeral_salt,
      ephemeral_expiry,
      idToken,
      jwt,
      domain,
    );

    print('Proof: ${proof?.proof} Error: ${proof?.error}');
    if (proof == null) {
      throw Exception('Proof is null: ${proof?.error}');
    } else {
      return proof.proof;
    }
  } catch (e) {
    print('Error creating message: $e');
  }
}

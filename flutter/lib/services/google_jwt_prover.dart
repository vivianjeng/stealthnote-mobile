import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:typed_data';
import '../models/signed_message.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

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
  final ephemeralPubkeyHash =
      "622618718926420486498127001071856504322492650656283936596477869965459887546";
  // final expiry = "2025-05-07T09:07:57.379Z";
  final expiry = "1746608877";
  final privateKey =
      "39919031573819484966641096195810516976016707561507350566056652693882791321787";
  final publicKey =
      "2162762795874508908128591380947689712526020850672181221274190323882846535333";
  final salt =
      "646645587996092179008704451306999156519169540151959619716525865713892520";

  try {
    final proof = await moproFlutterPlugin.proveJwt(
      srsPath,
      publicKey,
      salt,
      expiry,
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

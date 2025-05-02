import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:typed_data';

Uint8List extractProof(Uint8List result, int publicInputsLen) {
  final offset = 4 + publicInputsLen;
  return result.sublist(offset);
}

Future<void> createMembership(
  String ephemeralPubkey,
  String ephemeralPubkeyExpiry,
  String groupId,
  String provider,
  Uint8List proof,
  Map<String, dynamic> proofArgs,
) async {
  final url = Uri.parse(
    'https://ac1f-125-229-173-139.ngrok-free.app/api/memberships',
  ); // replace with your server

  final publicInputsLen = 2720;
  Uint8List rawProof = extractProof(proof, publicInputsLen);

  final response = await http.post(
    url,
    headers: {'Content-Type': 'application/json'},
    body: jsonEncode({
      'ephemeralPubkey': ephemeralPubkey,
      'ephemeralPubkeyExpiry': ephemeralPubkeyExpiry,
      'groupId': groupId,
      'provider': provider,
      'proof': rawProof, // Make sure this is a List<int> or List<String>
      'proofArgs': proofArgs, // Map<String, dynamic>
    }),
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    final errorMessage = response.body;
    print('Call to /memberships API failed: $errorMessage');
    throw Exception('Call to /memberships API failed');
  }
}

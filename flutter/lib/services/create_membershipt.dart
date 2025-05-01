import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:typed_data';
Future<void> createMembership(
  String ephemeralPubkey,
  String ephemeralPubkeyExpiry,
  String groupId,
  String provider,
  Uint8List proof,
  Map<String, dynamic> proofArgs,
) async {
  final url = Uri.parse(
    'https://stealthnote-pi.vercel.app/api/memberships',
  ); // replace with your server

  final response = await http.post(
    url,
    headers: {'Content-Type': 'application/json'},
    body: jsonEncode({
      'ephemeralPubkey': ephemeralPubkey,
      'ephemeralPubkeyExpiry': ephemeralPubkeyExpiry,
      'groupId': groupId,
      'provider': provider,
      'proof': proof, // Make sure this is a List<int> or List<String>
      'proofArgs': proofArgs, // Map<String, dynamic>
    }),
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    final errorMessage = response.body;
    print('Call to /memberships API failed: $errorMessage');
    throw Exception('Call to /memberships API failed');
  }
}

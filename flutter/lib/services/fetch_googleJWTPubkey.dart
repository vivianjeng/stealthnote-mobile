import 'dart:convert';
import 'package:http/http.dart' as http;

/// Fetches a Google public key by its key ID (kid)
Future<Map<String, dynamic>?> fetchGooglePublicKey(String keyId) async {
  if (keyId.isEmpty) {
    return null;
  }

  final response = await http.get(
    Uri.parse('https://www.googleapis.com/oauth2/v3/certs'),
  );

  if (response.statusCode != 200) {
    print('Failed to fetch Google public keys: ${response.statusCode}');
    return null;
  }

  final data = json.decode(response.body);
  final keys = data['keys'] as List<dynamic>;

  final key = keys.firstWhere(
    (k) => k['kid'] == keyId,
    orElse: () {
      print('Google public key with id $keyId not found');
      return null;
    },
  );

  return key as Map<String, dynamic>?;
}

import 'dart:convert';
import 'package:http/http.dart' as http;

import 'generate_ephemeral_key.dart';

Future<bool> toggleLike(String messageId, bool like) async {
  try {
    // Replace with your actual pubkey getter
    final ephemeralKey = await getEphemeralKey();

    // Decode the JSON string
    Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
    final ephemeralPubkey = ephemeralKeyObj['public_key'];

    final url = Uri.parse(
      'https://008f-125-229-173-139.ngrok-free.app/api/likes',
    );

    final response = await http.post(
      url,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer $ephemeralPubkey',
      },
      body: jsonEncode({
        'messageId': messageId,
        'like': like,
      }),
    );

    if (response.statusCode != 200) {
      final errorMessage = response.body;
      print('Call to /likes API failed: $errorMessage');
      throw Exception('Call to /likes API failed');
    } else {
      print(response.body);
    }

    final data = jsonDecode(response.body);
    return data['liked'] as bool;
  } catch (error) {
    print('Error toggling like: $error');
    rethrow;
  }
}

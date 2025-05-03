import 'dart:convert';
import 'package:http/http.dart' as http;
import './generate_ephemeral_key.dart';

Future<dynamic> fetchMessages() async {
  final url = Uri.parse(
    'https://ac1f-125-229-173-139.ngrok-free.app/api/messages?limit=5',
  );

  try {
    final response = await http.get(url);

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data;
    } else {
      print('Error: ${response.statusCode}');
    }
  } catch (e) {
    print('Failed to fetch messages: $e');
  }
}

Future<dynamic> fetchMessage(String id, bool isInternal) async {
  final headers = <String, String>{'Content-Type': 'application/json'};

  if (isInternal) {
    final pubkey = getEphemeralKey();
    if (pubkey == null) {
      throw Exception('No public key found');
    }
    headers['Authorization'] = 'Bearer $pubkey';
  }

  final response = await http.get(
    Uri.parse('https://ac1f-125-229-173-139.ngrok-free.app/api/messages/$id'),
    headers: headers,
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw Exception('Call to /messages/$id API failed: ${response.body}');
  }

  final message = jsonDecode(response.body);
  return message;
}

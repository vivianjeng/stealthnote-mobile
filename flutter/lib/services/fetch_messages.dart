import 'dart:convert';
import 'package:http/http.dart' as http;
import './generate_ephemeral_key.dart';

Future<dynamic> fetchMessages({
  required int limit,
  String? groupId,
  bool isInternal = false,
  int? afterTimestamp,
  int? beforeTimestamp,
}) async {
  final queryParams = {
    'limit': limit.toString(),
    if (groupId != null) 'groupId': groupId,
    if (isInternal) 'isInternal': 'true',
    if (afterTimestamp != null) 'afterTimestamp': afterTimestamp.toString(),
    if (beforeTimestamp != null) 'beforeTimestamp': beforeTimestamp.toString(),
  };

  final url = Uri.parse(
    'https://008f-125-229-173-139.ngrok-free.app/api/messages',
  ).replace(queryParameters: queryParams);

  try {
    final headers = <String, String>{
      'Content-Type': 'application/json',
      'Cache-Control': 'no-cache',
      'Pragma': 'no-cache'
    };

    if (isInternal) {
      final ephemeralKey = await getEphemeralKey();

      // Decode the JSON string
      Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
      final ephemeralPubkey = ephemeralKeyObj['public_key'];
      if (ephemeralPubkey == null) {
        throw Exception('No public key found');
      }
      headers['Authorization'] = 'Bearer $ephemeralPubkey';
    }
    final response = await http.get(
      url,
      headers: headers,
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data;
    } else {
      print('Error: ${response.statusCode} ${response.body}');
    }
  } catch (e) {
    print('Failed to fetch messages: $e');
  }
}

Future<dynamic> fetchMessage(String id, bool isInternal) async {
  final headers = <String, String>{
    'Content-Type': 'application/json',
    'Cache-Control': 'no-cache',
    'Pragma': 'no-cache'
  };

  if (isInternal) {
    final ephemeralKey = await getEphemeralKey();

    // Decode the JSON string
    Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
    final ephemeralPubkey = ephemeralKeyObj['public_key'];
    if (ephemeralPubkey == null) {
      throw Exception('No public key found');
    }
    headers['Authorization'] = 'Bearer $ephemeralPubkey';
  }

  final response = await http.get(
    Uri.parse('https://008f-125-229-173-139.ngrok-free.app/api/messages/$id'),
    headers: headers,
  );

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw Exception('Call to /messages/$id API failed: ${response.body}');
  }

  final message = jsonDecode(response.body);
  return message;
}

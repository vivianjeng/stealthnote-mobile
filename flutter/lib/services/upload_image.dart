import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'package:flutter_dotenv/flutter_dotenv.dart';

Future<String?> uploadToImgur(File imageFile) async {
  // Load the Client ID from .env file
  final clientId = dotenv.env['IMGUR_CLIENT_ID'];

  if (clientId == null || clientId.isEmpty) {
    print('IMGUR_CLIENT_ID not found in .env file or is empty.');
    return null;
  }

  var request = http.MultipartRequest(
    'POST',
    Uri.parse('https://api.imgur.com/3/image'),
  );
  request.headers['Authorization'] = 'Client-ID $clientId';
  request.files.add(
    await http.MultipartFile.fromPath(
      'image',
      imageFile.path,
    ),
  );

  try {
    final response = await request.send();
    if (response.statusCode == 200) {
      final responseBody = await response.stream.bytesToString();
      final jsonResponse = jsonDecode(responseBody);
      return jsonResponse['data']?['link'];
    } else {
      print('Imgur upload failed: ${response.statusCode}');
      print('Response body: ${await response.stream.bytesToString()}');
      return null;
    }
  } catch (e) {
    print('Error uploading to Imgur: $e');
    return null;
  }
} 
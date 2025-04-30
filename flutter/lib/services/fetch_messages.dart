import 'dart:convert';
import 'package:http/http.dart' as http;

Future<dynamic> fetchMessages() async {
  final url = Uri.parse('https://stealthnote.xyz/api/messages?limit=5');

  try {
    final response = await http.get(url);

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      print('Messages: $data');
      return data;
      // Handle the JSON data (e.g. parse to a model)
    } else {
      print('Error: ${response.statusCode}');
    }
  } catch (e) {
    print('Failed to fetch messages: $e');
  }
}

import 'package:flutter/material.dart';
import 'package:timeago/timeago.dart' as timeago;

class Message {
  final String org;
  final DateTime time;
  final String body;

  Message({required this.org, required this.time, required this.body});
}

class MessageCard extends StatelessWidget {
  final Message msg;
  const MessageCard(this.msg);

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                CircleAvatar(child: Icon(Icons.account_circle)),
                const SizedBox(width: 8),
                Expanded(child: Text('Someone from ${msg.org}')),
                Text(timeago.format(msg.time)),
              ],
            ),
            const SizedBox(height: 12),
            Text(msg.body),
            const SizedBox(height: 8),
            Row(
              children: [
                Icon(Icons.thumb_up_alt_outlined, size: 16),
                const SizedBox(width: 4),
                Text('0'),
                const Spacer(),
                TextButton(onPressed: () {}, child: Text('Verify'))
              ],
            ),
          ],
        ),
      ),
    );
  }
}

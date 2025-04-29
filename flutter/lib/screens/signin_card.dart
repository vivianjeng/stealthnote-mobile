import 'package:flutter/material.dart';
import '../services/jwt_prover.dart';

class SignInCard extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Text("What's happening behind the scenes at your company?",
                style: TextStyle(fontSize: 16)),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(
                    'Sign in with your Google work account to anonymously post as "Someone from your company".',
                    style: TextStyle(fontStyle: FontStyle.italic, fontSize: 14),
                  ),
                ),
                IconButton(
                  icon: Image.asset('assets/google.png', width: 36, height: 36),
                  onPressed: () {}, // Add Google sign-in logic here
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}


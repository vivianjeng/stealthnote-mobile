class SignedMessage {
  final String id;
  final String timestamp;
  final String text;
  final int likes = 0;
  final String ephemeralPubkey;
  final String signature;
  final bool internal;
  final String anonGroupId;
  final String anonGroupProvider = 'google-oauth';
  final String ephemeralPubkeyExpiry;

  SignedMessage({
    required this.id,
    required this.timestamp,
    required this.text,
    required this.ephemeralPubkey,
    required this.signature,
    required this.internal,
    required this.anonGroupId,
    required this.ephemeralPubkeyExpiry,
  });

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'timestamp': timestamp,
      'text': text,
      'likes': likes,
      'internal': internal,
      'anonGroupId': anonGroupId,
      'anonGroupProvider': anonGroupProvider,
      'ephemeralPubkeyExpiry': ephemeralPubkeyExpiry,
      'ephemeralPubkey': ephemeralPubkey,
      'signature': signature,
    };
  }
}

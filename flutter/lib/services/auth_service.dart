import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'dart:convert';

import 'fetch_googleJWTpubKey.dart';
import 'google_jwt_prover.dart';

Map<String, dynamic> parseJwtHeader(String? idToken) {
  if (idToken == null) {
    throw FormatException('Invalid ID token');
  }
  final parts = idToken.split('.');
  if (parts.length != 3) {
    throw FormatException('Invalid ID token');
  }

  final headerBase64 = base64Url.normalize(parts[0]);
  final headerString = utf8.decode(base64Url.decode(headerBase64));
  return json.decode(headerString);
}

Map<String, dynamic> parseJwtPayload(String? idToken) {
  if (idToken == null) {
    throw FormatException('Invalid ID token');
  }
  final parts = idToken.split('.');
  if (parts.length != 3) {
    throw FormatException('Invalid ID token');
  }

  final payloadBase64 = base64Url.normalize(parts[1]);
  final payloadString = utf8.decode(base64Url.decode(payloadBase64));
  return json.decode(payloadString);
}

class AuthService {
  final FirebaseAuth _auth = FirebaseAuth.instance;
  final GoogleSignIn _googleSignIn = GoogleSignIn();

  // Get the current user
  User? getCurrentUser() {
    return _auth.currentUser;
  }

  // Stream of auth state changes
  Stream<User?> get authStateChanges => _auth.authStateChanges();

  String sliceEmail(dynamic email) {
    return email.substring(email.indexOf('@') + 1);
  }

  Future<GoogleSignInAuthentication> getGoogleAuth() async {
    try {
      final GoogleSignInAccount? googleUser = await _googleSignIn.signIn();

      // If user cancels the sign-in process
      if (googleUser == null) {
        throw Exception('Google sign in failed');
      }

      return await googleUser.authentication;
    } catch (e) {
      print('Error getting Google authentication: $e');
      rethrow; // Rethrow to let the UI layer handle the error
    }
  }

  Future<OAuthCredential?> getGoogleCredential(
      GoogleSignInAuthentication googleAuth) async {
    try {
      return GoogleAuthProvider.credential(
        accessToken: googleAuth.accessToken,
        idToken: googleAuth.idToken,
      );
    } catch (e) {
      print('Error getting Google credential: $e');
      rethrow; // Rethrow to let the UI layer handle the error
    }
  }

  // Sign in with Google
  Future<UserCredential?> signInWithGoogle(OAuthCredential? credential) async {
    try {
      if (credential == null) {
        throw Exception('Google credential is null');
      }
      // // Begin Google Sign-In process
      // final GoogleSignInAccount? googleUser = await _googleSignIn.signIn();

      // // If user cancels the sign-in process
      // if (googleUser == null) {
      //   return null;
      // }

      // // Obtain auth details from the Google Sign-In
      // final GoogleSignInAuthentication googleAuth =
      //     await googleUser.authentication;

      // // Create a credential from the Google Sign-In details
      // final OAuthCredential credential = GoogleAuthProvider.credential(
      //   accessToken: googleAuth.accessToken,
      //   idToken: googleAuth.idToken,
      // );

      // final idToken = googleAuth.idToken;
      // final header = parseJwtHeader(idToken);
      // final payload = parseJwtPayload(idToken);

      // final googlePublicKey = await fetchGooglePublicKey(header['kid']);
      // print('idToken: $idToken');
      // print('Google Public Key: $googlePublicKey');
      // generateJwtProof(
      //   googlePublicKey.toString(),
      //   idToken,
      //   sliceEmail(payload['email']),
      // );

      // Sign in to Firebase with the Google credential
      return await _auth.signInWithCredential(credential);
    } catch (e) {
      print('Error signing in with Google: $e');
      rethrow; // Rethrow to let the UI layer handle the error
    }
  }

  // Sign out
  Future<void> signOut() async {
    try {
      await _googleSignIn.signOut();
      await _auth.signOut();
    } catch (e) {
      print('Error signing out: $e');
      rethrow;
    }
  }

  // Get current user with Google Sign In
  Future<User?> currentUser() async {
    try {
      final GoogleSignInAccount? account = await _googleSignIn.signIn();
      if (account == null) {
        return null;
      }

      final GoogleSignInAuthentication authentication =
          await account.authentication;

      final OAuthCredential credential = GoogleAuthProvider.credential(
        idToken: authentication.idToken,
        accessToken: authentication.accessToken,
      );

      final UserCredential authResult = await _auth.signInWithCredential(
        credential,
      );
      final User? user = authResult.user;

      return user;
    } catch (e) {
      print('Error getting current user: $e');
      rethrow;
    }
  }

  // Check if user is signed in
  bool isSignedIn() {
    return _auth.currentUser != null;
  }

  // Get user ID
  String? getUserId() {
    return _auth.currentUser?.uid;
  }

  // Get user email
  String? getUserEmail() {
    return _auth.currentUser?.email;
  }

  // Get user display name
  String? getUserDisplayName() {
    return _auth.currentUser?.displayName;
  }

  // Get user photo URL
  String? getUserPhotoUrl() {
    return _auth.currentUser?.photoURL;
  }
}

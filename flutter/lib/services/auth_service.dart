import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';

class AuthService {
  final FirebaseAuth _auth = FirebaseAuth.instance;
  final GoogleSignIn _googleSignIn = GoogleSignIn();

  // Get the current user
  User? getCurrentUser() {
    return _auth.currentUser;
  }

  // Stream of auth state changes
  Stream<User?> get authStateChanges => _auth.authStateChanges();

  // Sign in with Google
  Future<UserCredential?> signInWithGoogle() async {
    try {
      // Begin Google Sign-In process
      final GoogleSignInAccount? googleUser = await _googleSignIn.signIn();

      // If user cancels the sign-in process
      if (googleUser == null) {
        return null;
      }

      // Obtain auth details from the Google Sign-In
      final GoogleSignInAuthentication googleAuth =
          await googleUser.authentication;

      // Create a credential from the Google Sign-In details
      final OAuthCredential credential = GoogleAuthProvider.credential(
        accessToken: googleAuth.accessToken,
        idToken: googleAuth.idToken,
      );

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

import 'dart:convert';
import 'dart:typed_data';

import 'package:app/api/auth/password.dart';
import 'package:app/api/rest_api.dart';
import 'package:basic_utils/basic_utils.dart';
import 'package:pointycastle/export.dart';

class AuthRestClient {
  final RestClient api;
  final PKCS1Encoding encrypt;

  AuthRestClient(this.api, this.encrypt);

  static Future<AuthRestClient> create(RestClient api) async {
    final key = await api.getPasswordKey();
    final publicKey = CryptoUtils.rsaPublicKeyFromPemPkcs1(key.key);

    final encrypt = PKCS1Encoding(RSAEngine())
      ..init(true, PublicKeyParameter<RSAPublicKey>(publicKey));

    return AuthRestClient(api, encrypt);
  }

  Future<bool> authenticate(String email, String password) async {
    final encrypted = encrypt.process(
      Uint8List.fromList(utf8.encode(password)),
    );
    final body = PasswordAuthRequest(
      email: email,
      password: base64.encode(encrypted),
    );

    return (await api.authenticateWithPassword(body)).totp;
  }

  Future<void> requestSpecialAccess(String password) async {
    final encrypted = encrypt.process(
      Uint8List.fromList(utf8.encode(password)),
    );
    final body = PasswordAuthRequest(
      email: "",
      password: base64.encode(encrypted),
    );

    await api.requestSpecialAccess(body);
  }

  Future<void> confirmTotp(String code) async {
    final body = TotpConfirmRequest(code: code);
    await api.confirmTotp(body);
  }
}

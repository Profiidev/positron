import 'dart:convert';
import 'dart:typed_data';

import 'package:app/api/request.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:pointycastle/export.dart';
import 'package:basic_utils/basic_utils.dart';

part 'password.g.dart';

@JsonSerializable()
class KeyResponse {
  final String key;

  KeyResponse({required this.key});

  factory KeyResponse.fromJson(Map<String, dynamic> json) =>
      _$KeyResponseFromJson(json);
  Map<String, dynamic> toJson() => _$KeyResponseToJson(this);
}

@JsonSerializable()
class PasswordAuthRequest extends JsonSerializable {
  final String email;
  final String password;

  PasswordAuthRequest({required this.email, required this.password});

  factory PasswordAuthRequest.fromJson(Map<String, dynamic> json) =>
      _$PasswordAuthRequestFromJson(json);
  @override
  Map<String, dynamic> toJson() => _$PasswordAuthRequestToJson(this);
}

@JsonSerializable()
class PasswordAuthResponse extends JsonSerializable {
  final bool totp;

  PasswordAuthResponse({required this.totp});

  factory PasswordAuthResponse.fromJson(Map<String, dynamic> json) =>
      _$PasswordAuthResponseFromJson(json);
  @override
  Map<String, dynamic> toJson() => _$PasswordAuthResponseToJson(this);
}

class PasswordAuth extends RequestBase {
  final encryptor;

  PasswordAuth(this.encryptor);

  static Future<PasswordAuth> fromKey() async {
    var key = await _getKey();
    var publicKey = CryptoUtils.rsaPublicKeyFromPemPkcs1(key);

    var encryptor = PKCS1Encoding(RSAEngine())
      ..init(true, PublicKeyParameter<RSAPublicKey>(publicKey));

    return PasswordAuth(encryptor);
  }

  static Future<String> _getKey() async {
    var res = await RequestBase().getReqRes('/backend/auth/password/key');
    var keyRes = KeyResponse.fromJson(res);

    return keyRes.key;
  }

  Future<bool> authenticate(String email, String password) async {
    var encrypted = encryptor.process(
      Uint8List.fromList(utf8.encode(password)),
    );

    var body = PasswordAuthRequest(
      email: email,
      password: base64.encode(encrypted),
    );
    var res = await postReqBodyRes('/backend/auth/password/authenticate', body);
    var authResponse = PasswordAuthResponse.fromJson(res);

    return authResponse.totp;
  }
}

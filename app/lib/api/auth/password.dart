import 'package:json_annotation/json_annotation.dart';

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

@JsonSerializable()
class TotpConfirmRequest extends JsonSerializable {
  final String code;

  TotpConfirmRequest({required this.code});

  factory TotpConfirmRequest.fromJson(Map<String, dynamic> json) =>
      _$TotpConfirmRequestFromJson(json);
  @override
  Map<String, dynamic> toJson() => _$TotpConfirmRequestToJson(this);
}

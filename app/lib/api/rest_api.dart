import 'package:app/api/auth/passkey.dart';
import 'package:app/api/auth/password.dart';
import 'package:dio/dio.dart';
import 'package:passkeys/types.dart';
import 'package:retrofit/retrofit.dart';

part 'rest_api.g.dart';

@RestApi(baseUrl: "http://192.168.178.22:5175/backend")
abstract class RestClient {
  factory RestClient(Dio dio, {String? baseUrl}) = _RestClient;

  // Password Authentication
  @GET("/auth/password/key")
  Future<KeyResponse> getPasswordKey();

  @POST("/auth/password/authenticate")
  Future<PasswordAuthResponse> authenticateWithPassword(
    @Body() PasswordAuthRequest request,
  );

  // TOTP
  @POST("/auth/totp/confirm")
  Future<void> confirmTotp(@Body() TotpConfirmRequest request);

  // Passkey Authentication
  @GET("/auth/passkey/start_authentication")
  Future<PublicKeyCredentialRequest> startPasskeyAuth();

  @POST("/auth/passkey/finish_authentication")
  Future<void> finishPasskeyAuth(@Body() AuthenticateResponseType res);
}

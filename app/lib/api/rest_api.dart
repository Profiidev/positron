import 'package:app/api/auth/passkey.dart';
import 'package:app/api/auth/password.dart';
import 'package:dio/dio.dart';
import 'package:passkeys/types.dart';
import 'package:retrofit/retrofit.dart';

part 'rest_api.g.dart';

const baseUrl = "http://192.168.178.22:5175/";
const basePath = "backend";

@RestApi(baseUrl: "$baseUrl$basePath")
abstract class RestClient {
  factory RestClient(Dio dio, {String? baseUrl}) = _RestClient;

  // Password Authentication
  @GET("/auth/password/key")
  Future<KeyResponse> getPasswordKey();

  @POST("/auth/password/authenticate")
  Future<PasswordAuthResponse> authenticateWithPassword(
    @Body() PasswordAuthRequest request,
  );

  @POST("/auth/password/special_access")
  Future<void> requestSpecialAccess(@Body() PasswordAuthRequest request);

  // TOTP
  @POST("/auth/totp/confirm")
  Future<void> confirmTotp(@Body() TotpConfirmRequest request);

  // Passkey Authentication
  @GET("/auth/passkey/start_authentication")
  Future<PublicKeyCredentialRequest> startPasskeyAuth();

  @POST("/auth/passkey/finish_authentication/{id}")
  Future<void> finishPasskeyAuth(
    @Body() AuthenticateResponseType res,
    @Path("id") String id,
  );

  // Passkey Registration
  @GET("/auth/passkey/start_registration")
  Future<PublicKeyCredentialCreationOptions> startPasskeyRegistration();

  @POST("/auth/passkey/finish_registration")
  Future<void> finishPasskeyRegistration(
    @Body() PublicKeyCredentialCreationResponse response,
  );

  // Passkey Special Access
  @GET("/auth/passkey/start_special_access")
  Future<PublicKeyCredentialRequestOptions> startPasskeySpecialAccess();

  @POST("/auth/passkey/finish_special_access")
  Future<void> finishPasskeySpecialAccess(@Body() AuthenticateResponseType res);

  @GET("/auth/passkey/list")
  Future<List<Passkey>> listPasskeys();
}

import 'dart:convert';

import 'package:http/http.dart';
import 'package:json_annotation/json_annotation.dart';

var rootUri = "192.168.178.22:5173";
var useHttps = false;

class RequestBase {
  Uri _buildUri(String path) {
    if (useHttps) {
      return Uri.https(rootUri, path);
    } else {
      return Uri.http(rootUri, path);
    }
  }

  Future<Map<String, dynamic>> getReqRes(String path) async {
    var uri = _buildUri(path);
    var response = await get(uri);

    if (response.statusCode != 200) {
      throw Exception('Request failed with status: ${response.statusCode}');
    }

    var rawRes =
        jsonDecode(utf8.decode(response.bodyBytes)) as Map<String, dynamic>;
    return rawRes;
  }

  Future<void> postReqBody<T extends JsonSerializable>(
    String path,
    T body,
  ) async {
    var uri = _buildUri(path);

    var rawBody = json.encode(body.toJson());
    var response = await post(
      uri,
      headers: {'Content-Type': 'application/json'},
      body: rawBody,
    );

    if (response.statusCode != 200) {
      throw Exception('Request failed with status: ${response.statusCode}');
    }
  }

  Future<Map<String, dynamic>> postReqBodyRes<T extends JsonSerializable>(
    String path,
    T body,
  ) async {
    var uri = _buildUri(path);

    var rawBody = json.encode(body.toJson());
    var response = await post(
      uri,
      headers: {'Content-Type': 'application/json'},
      body: rawBody,
    );

    if (response.statusCode != 200) {
      throw Exception('Request failed with status: ${response.statusCode}');
    }

    var rawRes =
        jsonDecode(utf8.decode(response.bodyBytes)) as Map<String, dynamic>;
    return rawRes;
  }
}

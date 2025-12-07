import 'package:app/api/auth/client.dart';
import 'package:app/api/auth/passkey.dart';
import 'package:app/api/auth/password.dart';
import 'package:app/api/client.dart';
import 'package:app/api/rest_api.dart';
import 'package:app/state/auth.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:passkeys/authenticator.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

class SettingsState {
  final List<Passkey> passkeys;
  final bool specialLoading;
  String? errorMessage;
  AuthRestClient? passwordAuth;

  SettingsState({
    this.passkeys = const [],
    this.specialLoading = false,
    this.passwordAuth,
    this.errorMessage,
  });

  SettingsState copyWith({
    List<Passkey>? passkeys,
    bool? specialLoading,
    AuthRestClient? passwordAuth,
    String? errorMessage,
  }) {
    return SettingsState(
      passkeys: passkeys ?? this.passkeys,
      specialLoading: specialLoading ?? this.specialLoading,
      passwordAuth: passwordAuth ?? this.passwordAuth,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

class SettingsCubit extends Cubit<SettingsState> {
  final RestClient client;

  SettingsCubit(this.client) : super(SettingsState());
  SettingsCubit.create() : client = DioClient.create(), super(SettingsState());

  void getPasskeys() async =>
      emit(state.copyWith(passkeys: await client.listPasskeys()));

  void setSpecialLoading(bool loading) =>
      emit(state.copyWith(specialLoading: loading));

  void initializeAuth() async =>
      emit(state.copyWith(passwordAuth: await AuthRestClient.create(client)));

  void setErrorMessage(String message) {
    emit(state.copyWith(errorMessage: message));
  }
}

class SettingsPage extends StatelessWidget {
  final _nameController = TextEditingController();
  final _pwController = TextEditingController();

  SettingsPage({super.key});

  Future<void> _addPasskey(BuildContext context, String name) async {
    final cubit = context.read<SettingsCubit>();
    final auth = PasskeyAuthenticator();

    try {
      final options = await cubit.client.startPasskeyRegistration();
      final reg = await auth.register(options.publicKey);
      await cubit.client.finishPasskeyRegistration(
        PublicKeyCredentialCreationResponse(reg: reg, name: name),
      );
    } catch (e) {
      return;
    }
  }

  Future<bool> _confirmAccess(BuildContext context) async {
    final cubit = context.read<SettingsCubit>();
    final auth = cubit.state.passwordAuth;

    try {
      cubit.setSpecialLoading(true);
      await auth?.requestSpecialAccess(_pwController.text);
    } catch (e) {
      cubit.setErrorMessage("Password incorrect");
      return false;
    } finally {
      cubit.setSpecialLoading(false);
    }
    return true;
  }

  void addDialog(BuildContext context, SettingsState state) {
    showShadDialog(
      context: context,
      builder: (_) => ShadDialog(
        title: const Text("Add Passkey"),
        description: const Text("Enter name for new passkey"),
        actions: [
          ShadButton(
            onPressed: () async {
              await _addPasskey(context, _nameController.text);
              context.read<SettingsCubit>().getPasskeys();
            },
            leading: state.specialLoading
                ? SizedBox.square(
                    dimension: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: ShadTheme.of(
                        context,
                      ).colorScheme.primaryForeground,
                    ),
                  )
                : null,
            child: Text("Add"),
          ),
          ShadButton.secondary(
            onPressed: () {
              Navigator.of(context).pop();
            },
            child: Text("Cancel"),
          ),
        ],
        child: Container(
          width: 375,
          padding: EdgeInsets.symmetric(vertical: 20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.end,
            spacing: 16,
            children: [ShadInput(controller: _nameController)],
          ),
        ),
      ),
    );
  }

  void specialDialog(BuildContext context, SettingsState state) {
    showShadDialog(
      context: context,
      builder: (_) => ShadDialog(
        title: const Text("Confirm Access"),
        description: const Text("Confirm your access to continue"),
        actions: [
          ShadButton(
            onPressed: () async {
              if (await _confirmAccess(context)) {
                addDialog(context, state);
              }
            },
            child: Text("Confirm"),
          ),
          ShadButton.secondary(
            onPressed: () {
              Navigator.of(context).pop();
            },
            child: Text("Cancel"),
          ),
        ],
        child: Container(
          width: 375,
          padding: EdgeInsets.symmetric(vertical: 20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.end,
            spacing: 16,
            children: (() {
              var widgets = <Widget>[];

              if (state.errorMessage != null) {
                widgets.add(
                  Text(
                    state.errorMessage!,
                    style: TextStyle(
                      color: ShadTheme.of(context).colorScheme.destructive,
                    ),
                  ),
                );
              }

              widgets.add(
                ShadInput(controller: _pwController, obscureText: true),
              );

              return widgets;
            })(),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => SettingsCubit.create()
        ..getPasskeys()
        ..initializeAuth(),
      child: BlocBuilder<SettingsCubit, SettingsState>(
        builder: (context, state) {
          return Padding(
            padding: EdgeInsets.only(top: 40.0),
            child: Padding(
              padding: EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: (() {
                  var widgets = [
                    Text("Settings", style: ShadTheme.of(context).textTheme.h2),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          "Passkeys",
                          style: ShadTheme.of(context).textTheme.h4,
                        ),
                        ShadIconButton(
                          icon: Icon(LucideIcons.plus),
                          onPressed: () async {
                            if (await checkSpecialAccess()) {
                              addDialog(context, state);
                            } else {
                              specialDialog(context, state);
                            }
                          },
                        ),
                      ],
                    ),
                  ];

                  for (var passkey in state.passkeys) {
                    widgets.add(Text("- ${passkey.name}"));
                  }

                  return widgets;
                })(),
              ),
            ),
          );
        },
      ),
    );
  }
}

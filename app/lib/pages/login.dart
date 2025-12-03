import 'package:app/api/auth/client.dart';
import 'package:app/api/client.dart';
import 'package:app/state/auth.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

class LoginState {
  bool obscured = true;
  bool totp = false;
  AuthRestClient? passwordAuth;
  String errorMessage = "";
  bool loading = false;

  LoginState copyWith({
    bool? obscured,
    bool? totp,
    AuthRestClient? passwordAuth,
    String? errorMessage,
    bool? loading,
  }) {
    return LoginState()
      ..obscured = obscured ?? this.obscured
      ..totp = totp ?? this.totp
      ..passwordAuth = passwordAuth ?? this.passwordAuth
      ..errorMessage = errorMessage ?? this.errorMessage
      ..loading = loading ?? this.loading;
  }
}

class LoginCubit extends Cubit<LoginState> {
  LoginCubit() : super(LoginState());

  void toggleObscure() => emit(state.copyWith(obscured: !state.obscured));
  void requestTotp() => emit(state.copyWith(totp: true, loading: false));
  void setLoading(bool loading) => emit(state.copyWith(loading: loading));
  void setErrorMessage(String message) =>
      emit(state.copyWith(errorMessage: message));
  void initializeAuth() async => emit(
    state.copyWith(
      passwordAuth: await AuthRestClient.create(DioClient.create()),
    ),
  );
}

class LoginPage extends StatelessWidget {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => LoginCubit()..initializeAuth(),
      child: BlocBuilder<LoginCubit, LoginState>(
        builder: (context, state) {
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: LoginForm(state: state),
            ),
          );
        },
      ),
    );
  }
}

class LoginForm extends StatelessWidget {
  LoginForm({super.key, required this.state});

  final LoginState state;
  final formKey = GlobalKey<ShadFormState>();

  void _validateForm(BuildContext context) async {
    final cubit = context.read<LoginCubit>();

    if (state.passwordAuth == null) {
      cubit.setErrorMessage("Authentication not initialized");
      return;
    }

    final loggedInCubit = context.read<LoggedInCubit>();

    if (formKey.currentState!.saveAndValidate()) {
      cubit.setErrorMessage("");
      final formData = formKey.currentState!.value;
      if (state.totp) {
        final code = formData['totp'] as String;

        try {
          cubit.setLoading(true);
          await state.passwordAuth!.confirmTotp(code);
        } catch (e) {
          cubit.setErrorMessage("Invalid TOTP code");
          return;
        } finally {
          cubit.setLoading(false);
        }

        loggedInCubit.checkLoggedIn();
      } else {
        final email = formData['email'] as String;
        final password = formData['password'] as String;

        late bool totpRequired;
        try {
          cubit.setLoading(true);
          totpRequired = await state.passwordAuth!.authenticate(
            email,
            password,
          );
        } catch (e) {
          cubit.setErrorMessage("Invalid email or password");
          return;
        } finally {
          cubit.setLoading(false);
        }

        if (totpRequired) {
          if (!context.mounted) return;
          cubit.requestTotp();
        } else {
          loggedInCubit.checkLoggedIn();
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return ShadForm(
      key: formKey,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: () {
          var widgets = <Widget>[];

          if (!state.totp) {
            widgets.add(
              Padding(
                padding: const EdgeInsets.only(bottom: 8.0),
                child: ShadInputFormField(
                  id: "email",
                  label: const Text("Email"),
                  placeholder: const Text("Enter your email"),
                  leading: const Padding(
                    padding: EdgeInsets.all(4.0),
                    child: Icon(LucideIcons.user),
                  ),
                  validator: (v) {
                    if (v.isEmpty) {
                      return 'Please enter your username';
                    }
                    return null;
                  },
                ),
              ),
            );

            widgets.add(
              ShadInputFormField(
                id: "password",
                label: const Text("Password"),
                placeholder: const Text("Enter your password"),
                obscureText: state.obscured,
                leading: const Padding(
                  padding: EdgeInsets.all(4.0),
                  child: Icon(LucideIcons.lock),
                ),
                trailing: ShadButton(
                  width: 24,
                  height: 24,
                  padding: EdgeInsets.zero,
                  child: Icon(
                    state.obscured ? LucideIcons.eyeOff : LucideIcons.eye,
                  ),
                  onPressed: () {
                    context.read<LoginCubit>().toggleObscure();
                  },
                ),
                validator: (v) {
                  if (v.isEmpty) {
                    return 'Please enter your password';
                  }
                  return null;
                },
              ),
            );
          } else {
            widgets.add(
              ShadInputOTPFormField(
                id: 'totp',
                maxLength: 6,
                label: const Text('TOTP Code'),
                validator: (v) {
                  if (v.contains(' ')) {
                    return 'Please enter a valid TOTP code';
                  }
                  return null;
                },
                children: [
                  ShadInputOTPGroup(
                    children: [
                      ShadInputOTPSlot(),
                      ShadInputOTPSlot(),
                      ShadInputOTPSlot(),
                    ],
                  ),
                  Icon(size: 24, LucideIcons.dot),
                  ShadInputOTPGroup(
                    children: [
                      ShadInputOTPSlot(),
                      ShadInputOTPSlot(),
                      ShadInputOTPSlot(),
                    ],
                  ),
                ],
              ),
            );
          }

          if (state.errorMessage.isNotEmpty) {
            widgets.add(
              Container(
                width: double.infinity,
                child: Padding(
                  padding: const EdgeInsets.only(top: 16.0),
                  child: Text(
                    state.errorMessage,
                    style: TextStyle(
                      color: ShadTheme.of(context).colorScheme.destructive,
                    ),
                  ),
                ),
              ),
            );
          }

          widgets.add(
            Padding(
              padding: const EdgeInsets.only(top: 16.0),
              child: ShadButton(
                onPressed: () => _validateForm(context),
                width: double.infinity,
                leading: state.passwordAuth == null || state.loading
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
                child: state.totp
                    ? const Text("Submit")
                    : const Text("Sign In"),
              ),
            ),
          );

          return widgets;
        }(),
      ),
    );
  }
}

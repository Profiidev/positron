import 'package:app/api/auth/client.dart';
import 'package:app/api/client.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

class LoginState {
  bool obscured = true;
  bool totp = false;
  AuthRestClient? passwordAuth;

  LoginState copyWith({
    bool? obscured,
    bool? totp,
    AuthRestClient? passwordAuth,
  }) {
    return LoginState()
      ..obscured = obscured ?? this.obscured
      ..totp = totp ?? this.totp
      ..passwordAuth = passwordAuth ?? this.passwordAuth;
  }
}

abstract class LoginEvent {}

final class LoginInit extends LoginEvent {}

final class LoginObscureToggled extends LoginEvent {}

final class LoginTotpRequested extends LoginEvent {}

class LoginBloc extends Bloc<LoginEvent, LoginState> {
  LoginBloc() : super(LoginState()) {
    on<LoginInit>((event, emit) async {
      emit(
        state.copyWith(
          passwordAuth: await AuthRestClient.create(DioClient.create()),
        ),
      );
    });

    on<LoginObscureToggled>((event, emit) {
      emit(state.copyWith(obscured: !state.obscured));
    });

    on<LoginTotpRequested>((event, emit) {
      emit(state.copyWith(totp: true));
    });
  }
}

class LoginPage extends StatelessWidget {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => LoginBloc()..add(LoginInit()),
      child: BlocBuilder<LoginBloc, LoginState>(
        builder: (context, state) {
          return Center(child: LoginForm(state: state));
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
    if (state.passwordAuth == null) {
      return;
    }

    if (formKey.currentState!.saveAndValidate()) {
      final formData = formKey.currentState!.value;
      if (state.totp) {
        final code = formData['totp'] as String;

        await state.passwordAuth!.confirmTotp(code);

        print("TOTP confirmed, login successful");
      } else {
        final email = formData['email'] as String;
        final password = formData['password'] as String;

        final totpRequired = await state.passwordAuth!.authenticate(
          email,
          password,
        );

        if (totpRequired) {
          print("TOTP required for this user");
          if (!context.mounted) return;
          context.read<LoginBloc>().add(LoginTotpRequested());
        } else {
          print("Login successful");
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
              ShadInputFormField(
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
                    context.read<LoginBloc>().add(LoginObscureToggled());
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

          widgets.add(
            ShadButton(
              onPressed: () => _validateForm(context),
              leading: state.passwordAuth == null
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
              child: state.totp ? const Text("Submit") : const Text("Login"),
            ),
          );

          return widgets;
        }(),
      ),
    );
  }
}

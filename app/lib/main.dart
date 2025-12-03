import 'package:app/router.dart';
import 'package:flutter/material.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

void main() {
  runApp(const Positron());
}

class Positron extends StatelessWidget {
  const Positron({super.key});

  @override
  Widget build(BuildContext context) {
    return ShadApp.router(
      title: 'Flutter Demo',
      theme: ShadThemeData(
        brightness: Brightness.light,
        colorScheme: const ShadBlueColorScheme.light(),
      ),
      darkTheme: ShadThemeData(
        brightness: Brightness.dark,
        colorScheme: const ShadBlueColorScheme.dark(),
      ),
      routerConfig: router,
    );
  }
}

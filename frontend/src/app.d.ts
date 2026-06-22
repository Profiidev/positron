declare global {
  interface NavigatorUAData {
    readonly brands: { brand: string; version: string }[];
    readonly mobile: boolean;
    readonly platform: string;
  }

  interface Navigator {
    userAgentData?: NavigatorUAData;
  }
}

// oxlint-disable-next-line require-module-specifiers
export {};

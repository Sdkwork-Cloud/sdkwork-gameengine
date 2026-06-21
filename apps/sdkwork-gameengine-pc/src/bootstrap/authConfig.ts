import type { SdkworkAuthRuntimeConfig } from '@sdkwork/auth-pc-react';

export interface SdkworkGameenginePcAuthAppearanceConfig {
  asidePanelClassName?: string;
  bodyClassName?: string;
  contentContainerClassName?: string;
  pageClassName?: string;
  qrFrameClassName?: string;
  shellClassName?: string;
  slotProps?: {
    background?: { className?: string };
    page?: { className?: string };
    shell?: { className?: string };
  };
  theme?: Record<string, string>;
}

export type SdkworkGameenginePcAuthRuntimeConfig = SdkworkAuthRuntimeConfig;

const GAMES_VERIFICATION_POLICY = {
  emailCodeLoginEnabled: true,
  emailRegistrationVerificationRequired: false,
  phoneCodeLoginEnabled: true,
  phoneRegistrationVerificationRequired: false,
};

export function resolveSdkworkGameenginePcAuthRuntimeConfig(): SdkworkGameenginePcAuthRuntimeConfig {
  return {
    leftRailMode: 'qr-only',
    loginMethods: ['password', 'emailCode', 'phoneCode'],
    oauthLoginEnabled: false,
    oauthProviders: [],
    qrLoginEnabled: true,
    recoveryMethods: ['email', 'phone'],
    registerMethods: ['email', 'phone'],
    verificationPolicy: GAMES_VERIFICATION_POLICY,
  };
}

export function resolveSdkworkGameenginePcAuthAppearance(): SdkworkGameenginePcAuthAppearanceConfig {
  return {
    asidePanelClassName: 'sdkwork-gameengine-pc-auth-aside-panel',
    bodyClassName: 'sdkwork-gameengine-pc-auth-body',
    contentContainerClassName: 'sdkwork-gameengine-pc-auth-content',
    pageClassName: 'sdkwork-gameengine-pc-auth-page',
    qrFrameClassName: 'sdkwork-gameengine-pc-auth-qr-frame',
    shellClassName: 'sdkwork-gameengine-pc-auth-card-shell',
    slotProps: {
      background: {
        className: 'sdkwork-gameengine-pc-auth-background',
      },
      page: {
        className: 'sdkwork-gameengine-pc-auth-page',
      },
      shell: {
        className: 'sdkwork-gameengine-pc-auth-card-shell',
      },
    },
  };
}

export function resolveSdkworkGameenginePcAuthLocale(defaultLocale: string): string {
  return defaultLocale;
}

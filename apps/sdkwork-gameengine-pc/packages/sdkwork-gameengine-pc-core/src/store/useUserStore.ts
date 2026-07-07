import { create } from 'zustand';

export interface UserProfile {
  id: string;
  username: string;
  avatar: string;
  tenantId?: string;
  organizationId?: string;
  sessionId?: string;
}

export interface IamSessionProfileInput {
  userId: string;
  displayName?: string;
  avatarUrl?: string;
  tenantId?: string;
  organizationId?: string;
  sessionId?: string;
}

interface UserState {
  profile: UserProfile | null;
  clearIdentity: () => void;
  syncFromIamSession: (input: IamSessionProfileInput | null) => void;
}

function profileFromIamSession(input: IamSessionProfileInput): UserProfile {
  const displayName = input.displayName?.trim();
  const avatarUrl = input.avatarUrl?.trim();
  return {
    id: input.userId,
    username: displayName || input.userId,
    avatar:
      avatarUrl ||
      `https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(input.userId)}`,
    tenantId: input.tenantId,
    organizationId: input.organizationId,
    sessionId: input.sessionId,
  };
}

export const useUserStore = create<UserState>()((set) => ({
  profile: null,
  clearIdentity: () => set({ profile: null }),
  syncFromIamSession: (input) => {
    if (!input?.userId) {
      set({ profile: null });
      return;
    }
    set({ profile: profileFromIamSession(input) });
  },
}));

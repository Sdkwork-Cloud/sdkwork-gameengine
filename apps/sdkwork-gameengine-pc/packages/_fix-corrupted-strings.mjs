import fs from 'node:fs';
import path from 'node:path';

const packagesDir = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Z]:)/, '$1'));

const localeKeys = new Set();
for (const file of [
  'sdkwork-gameengine-pc-i18n/src/locales/en/common.ts',
  'sdkwork-gameengine-pc-i18n/src/locales/en/store.ts',
  'sdkwork-gameengine-pc-i18n/src/locales/en/ringmatch.ts',
  'sdkwork-gameengine-pc-i18n/src/locales/en/arena.ts',
  'sdkwork-gameengine-pc-i18n/src/locales/en/dashboard.ts',
  'sdkwork-gameengine-pc-i18n/src/locales/en/quiz.ts',
]) {
  const content = fs.readFileSync(path.join(packagesDir, file), 'utf8');
  for (const match of content.matchAll(/"([^"]+)":/g)) {
    localeKeys.add(match[1]);
  }
}

const englishFallbacks = {
  claws_desc: 'Manage and train your AI combat agents.',
  create_claw: 'Create Agent',
  status_idle: 'Idle',
  status_defending: 'Defending',
  status_training: 'Training',
  power: 'Power',
  training: 'Training...',
  train_claw: 'Train (500 Tokens)',
  go_defend: 'Go Defend',
  claw_name: 'Agent Name',
  claw_name_placeholder: 'Enter agent name...',
  confirm_create: 'Create',
  train_success: 'Training complete!',
  purchase_success: 'Purchase successful!',
  drive_your_agents: 'Drive Your Agents',
  custom_amount: 'Custom Amount',
  enter_custom_amount: 'Enter token amount to purchase',
  custom_amount_hint: 'Custom amounts do not include bonus tokens. Minimum 100 Tokens.',
  payment_method: 'Payment Method',
  pay_with_points: 'Pay with Points',
  balance: 'Balance',
  direct_purchase: 'Direct Purchase',
  support_wechat_alipay: 'WeChat / Alipay supported',
  order_summary: 'Order Summary',
  base_tokens: 'Base Tokens',
  total_receive: 'Total Receive',
  total_cost: 'Total Cost',
  confirm_purchase: 'Confirm Purchase',
  insufficient_points_hint: 'Insufficient points. Adjust amount or change payment method.',
  manage_your_assets: 'Manage Your Assets',
  wallet_desc: 'Top up for more points, or withdraw points earned in games.',
  total_balance: 'Total Balance (Points)',
  compute_tokens: 'Compute Balance',
  go_to_compute: 'Go to Compute Center',
  deposit: 'Deposit',
  withdraw: 'Withdraw',
  history: 'History',
  select_deposit_amount: 'Select Deposit Amount',
  custom_deposit: 'Custom Deposit (Points)',
  min_deposit_100: 'Minimum 100 points',
  payment_summary: 'Payment Summary',
  base_points: 'Base Points',
  min_deposit: 'Minimum deposit is 100 points',
  deposit_success: 'Deposit successful!',
  invalid_amount: 'Please enter a valid withdrawal amount',
  min_withdraw: 'Minimum withdrawal is 1000 points',
  withdraw_success: 'Withdrawal submitted. Expected within 24 hours.',
  bind_account_title: 'Bind Withdrawal Account',
  bind_account_desc: 'For your fund security, please bind a verified withdrawal account first.',
  bind_now: 'Bind Now',
  withdraw_amount: 'Withdrawal Amount (Points)',
  min_withdraw_placeholder: 'Minimum 1000 points',
  withdraw_all: 'Withdraw All',
  exchange_rate: 'Exchange rate: 100 points = ¥1',
  estimated_fiat: 'Estimated payout',
  payout_method: 'Payout Method',
  withdraw_notice_title: 'Withdrawal Notice',
  withdraw_notice_1: 'Withdrawal requests are processed within 24 hours.',
  withdraw_notice_2: 'A 5% processing fee applies.',
  withdraw_notice_3: 'Ensure your payout account is valid and bound.',
  confirm_withdraw: 'Confirm Withdrawal',
  no_transactions: 'No transactions yet',
  exchange: 'Exchange',
  success: 'Success',
  challenge_won: 'Challenge won!',
  challenge_lost: 'Challenge lost.',
  spectate: 'Spectate',
  all: 'All',
};

const files = [
  'sdkwork-gameengine-pc-arena/src/pages/AIArena.tsx',
  'sdkwork-gameengine-pc-claws/src/pages/ClawsManager.tsx',
  'sdkwork-gameengine-pc-compute/src/pages/ComputeCenter.tsx',
  'sdkwork-gameengine-pc-ringmatch/src/pages/RingMatch.tsx',
  'sdkwork-gameengine-pc-wallet/src/pages/Wallet.tsx',
];

function fixTCalls(content) {
  return content.replace(/t\(\s*'([^']+)'\s*,\s*'(?:[^'\\]|\\.)*?(?:\?|,|\)|$)/g, (match, key) => {
    if (localeKeys.has(key)) {
      return `t('${key}')`;
    }
    const fallback = englishFallbacks[key] ?? key.replace(/_/g, ' ');
    return `t('${key}', '${fallback.replace(/'/g, "\\'")}')`;
  });
}

function fixBrokenSpans(content) {
  return content
    .replace(/<span className="font-bold">[^<]*$/gm, (line) => {
      if (line.includes('支付宝')) return '<span className="font-bold">Alipay</span>';
      if (line.includes('微信支付')) return '<span className="font-bold">WeChat Pay</span>';
      if (line.includes('银行')) return '<span className="font-bold">Bank Card</span>';
      return line;
    });
}

for (const rel of files) {
  const filePath = path.join(packagesDir, rel);
  let content = fs.readFileSync(filePath, 'utf8');
  content = fixTCalls(content);
  content = fixBrokenSpans(content);
  content = content
    .replace(/avatar: "đ[^"]*$/gm, 'avatar: "🛡️",')
    .replace(/avatar: "đŸŚž"/g, 'avatar: "🦞"')
    .replace(/avatar: "đŸ§ "/g, 'avatar: "🧠"')
    .replace(/avatar: "đŸ¤–"/g, 'avatar: "🤖"')
    .replace(/avatar: "đŸŚž"/g, 'avatar: "🦞"')
    .replace(/\? "đŸŚž"\s*:\s*newRing\.creatorType === "Team" \? "đ[^"]*$/gm, '? "🦞" : newRing.creatorType === "Team" ? "🛡️"')
    .replace(/showToast\(t\('copy'\) \+ " " \+ t\('success_msg'\)[^;]+;/g, "showToast(`${t('copy')} copied successfully!`, 'success');")
    .replace(/<span className="text-4xl">\?\?<\/span>/g, '<span className="text-4xl">🦞</span>')
    .replace(/>\?\?\s*<\/div>/g, '>🦞</div>')
    .replace(/<span>\?\?<\/span>/g, '<span>🦞</span>')
    .replace(/ÂĽ/g, '¥')
    .replace(/Â·/g, '·');
  fs.writeFileSync(filePath, content, 'utf8');
  console.log('fixed', rel);
}

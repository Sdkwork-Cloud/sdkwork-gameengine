import React, { useState } from "react";
import { motion } from "motion/react";
import { CreditCard, Database, Zap, ArrowRight, ShieldCheck, Gem, ArrowDownToLine, ArrowUpFromLine } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUserStore } from "sdkwork-gameengine-pc-core";

export default function Recharge() {
  const { t } = useTranslation();
  const { profile, addPoints, deductPoints } = useUserStore();
  const [selectedAmount, setSelectedAmount] = useState<number | null>(null);
  const [isRecharging, setIsRecharging] = useState(false);
  const [mode, setMode] = useState<"recharge" | "withdraw">("recharge");
  const [withdrawAmount, setWithdrawAmount] = useState<string>("");

  const rechargeOptions = [
    { id: 1, points: 1000, price: "¥10", bonus: 0, popular: false },
    { id: 2, points: 5000, price: "¥48", bonus: 200, popular: true },
    { id: 3, points: 10000, price: "¥90", bonus: 1000, popular: false },
    { id: 4, points: 50000, price: "¥400", bonus: 10000, popular: false },
  ];

  const handleRecharge = () => {
    if (mode === "recharge") {
      if (!selectedAmount) return;
      const opt = rechargeOptions.find(o => o.id === selectedAmount);
      if (!opt) return;

      setIsRecharging(true);
      setTimeout(() => {
        addPoints(opt.points + opt.bonus);
        setSelectedAmount(null);
        setIsRecharging(false);
        alert(t('recharge_success', '充值成�?));
      }, 1000);
    } else {
      const amount = parseInt(withdrawAmount);
      if (isNaN(amount) || amount <= 0) {
        alert(t('invalid_amount', '请输入有效的提现金额'));
        return;
      }
      if (amount < 1000) {
        alert(t('min_withdraw', '最低提现额度为 1000 积分'));
        return;
      }
      if (profile && profile.points < amount) {
        alert(t('insufficient_points', '积分不足'));
        return;
      }

      setIsRecharging(true);
      setTimeout(() => {
        const success = deductPoints(amount);
        if (success) {
          setWithdrawAmount("");
          alert(t('withdraw_success', '提现申请已提交，预计 24 小时内到�?));
        }
        setIsRecharging(false);
      }, 1000);
    }
  };

  return (
    <div className="space-y-8 pb-12 max-w-5xl mx-auto">
      {/* Header */}
      <div className="text-center space-y-4 py-12 relative">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-blue-600/20 rounded-full blur-[100px] pointer-events-none"></div>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="relative z-10"
        >
          <div className="inline-flex items-center justify-center space-x-2 px-4 py-1.5 rounded-full bg-blue-100 dark:bg-blue-500/10 border border-blue-200 dark:border-blue-500/20 text-blue-600 dark:text-blue-400 text-sm font-bold mb-6">
            <CreditCard size={16} />
            <span>{t('wallet_center', '钱包中心')}</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-black text-zinc-900 dark:text-white tracking-tight mb-4">
            {t('manage_your_assets', '管理你的资产')}
          </h1>
          <p className="text-zinc-500 dark:text-zinc-400 text-lg max-w-2xl mx-auto font-medium">
            {t('wallet_desc', '充值获取更多积分，或将你在游戏中赢得的积分提现�?)}
          </p>
        </motion.div>
      </div>

      {/* Current Balance */}
      <div className="bg-gradient-to-r from-zinc-900 to-zinc-800 rounded-3xl p-8 border border-zinc-700 shadow-2xl relative overflow-hidden z-10 flex flex-col md:flex-row items-center justify-between">
        <div className="absolute top-0 right-0 w-64 h-64 bg-blue-500/20 rounded-full blur-[80px] -translate-y-1/2 translate-x-1/3 pointer-events-none"></div>
        
        <div className="flex items-center space-x-6 z-10 mb-6 md:mb-0">
          <div className="w-20 h-20 bg-blue-500/20 rounded-2xl flex items-center justify-center border border-blue-500/30 shadow-[0_0_30px_rgba(59,130,246,0.3)]">
            <Database className="text-blue-400" size={40} />
          </div>
          <div>
            <p className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-1">{t('current_balance', '当前积分余额')}</p>
            <p className="text-4xl font-black text-white flex items-center">
              {profile?.points?.toLocaleString() || 0} <span className="text-lg text-zinc-500 ml-2 font-medium">pts</span>
            </p>
          </div>
        </div>
        
        <div className="flex space-x-4 z-10">
          <div className="bg-black/40 backdrop-blur-md px-6 py-4 rounded-2xl border border-white/10 flex flex-col items-center">
            <Gem className="text-purple-400 mb-2" size={24} />
            <span className="text-xs text-zinc-400 font-bold">{t('vip_level', 'VIP 等级')}</span>
            <span className="text-lg font-black text-white">{profile?.vipLevel !== 'none' ? `VIP ${profile?.vipLevel.toUpperCase()}` : 'None'}</span>
          </div>
          <div className="bg-black/40 backdrop-blur-md px-6 py-4 rounded-2xl border border-white/10 flex flex-col items-center">
            <ShieldCheck className="text-emerald-400 mb-2" size={24} />
            <span className="text-xs text-zinc-400 font-bold">{t('secure_payment', '安全支付')}</span>
            <span className="text-lg font-black text-white">100%</span>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="mt-12 relative z-10">
        <div className="flex flex-col sm:flex-row items-center justify-between mb-6 gap-4">
          <h2 className="text-2xl font-black text-zinc-900 dark:text-white flex items-center">
            {mode === "recharge" ? <ArrowDownToLine className="mr-2 text-blue-500" /> : <ArrowUpFromLine className="mr-2 text-emerald-500" />}
            {mode === "recharge" ? t('select_amount', '选择充值金�?) : t('withdraw_points', '积分提现')}
          </h2>
          <div className="flex bg-zinc-200 dark:bg-zinc-800 p-1 rounded-xl">
            <button
              onClick={() => { setMode("recharge"); setSelectedAmount(null); }}
              className={`px-6 py-2 rounded-lg font-bold text-sm transition-all ${
                mode === "recharge" 
                  ? "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-white shadow-sm" 
                  : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
              }`}
            >
              {t('recharge', '充�?)}
            </button>
            <button
              onClick={() => { setMode("withdraw"); setSelectedAmount(null); }}
              className={`px-6 py-2 rounded-lg font-bold text-sm transition-all ${
                mode === "withdraw" 
                  ? "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-white shadow-sm" 
                  : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
              }`}
            >
              {t('withdraw', '提现')}
            </button>
          </div>
        </div>

        {mode === "recharge" ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {rechargeOptions.map((opt, i) => (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.1 }}
                key={opt.id}
                onClick={() => setSelectedAmount(opt.id)}
                className={`relative bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border-2 transition-all cursor-pointer overflow-hidden flex flex-col items-center text-center ${
                  selectedAmount === opt.id 
                    ? "border-blue-500 shadow-[0_0_30px_rgba(59,130,246,0.2)] scale-105" 
                    : "border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50 hover:scale-105"
                }`}
              >
                {opt.popular && (
                  <div className="absolute top-0 right-0 bg-blue-500 text-white text-[10px] font-black px-3 py-1 rounded-bl-xl uppercase tracking-wider">
                    {t('best_value', '最划算')}
                  </div>
                )}
                
                <Database size={48} className={`mb-4 ${selectedAmount === opt.id ? "text-blue-500" : "text-blue-500/50"}`} />
                
                <h3 className="text-3xl font-black text-zinc-900 dark:text-white mb-1">{opt.points.toLocaleString()}</h3>
                <p className="text-sm font-bold text-zinc-500 mb-4">Points</p>
                
                {opt.bonus > 0 && (
                  <div className="text-xs font-bold text-emerald-500 bg-emerald-500/10 px-3 py-1 rounded-full mb-4">
                    +{opt.bonus} {t('bonus', '赠�?)}
                  </div>
                )}
                
                <div className="w-full h-px bg-zinc-200 dark:bg-zinc-800 my-4 mt-auto"></div>
                
                <div className="text-2xl font-black text-zinc-900 dark:text-white">
                  {opt.price}
                </div>
              </motion.div>
            ))}
          </div>
        ) : (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-8 rounded-3xl border border-zinc-200 dark:border-zinc-800 max-w-2xl mx-auto"
          >
            <div className="mb-6">
              <label className="block text-sm font-bold text-zinc-700 dark:text-zinc-300 mb-2">
                {t('withdraw_amount', '提现积分数量')}
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                  <Database className="text-zinc-400" size={20} />
                </div>
                <input
                  type="number"
                  value={withdrawAmount}
                  onChange={(e) => setWithdrawAmount(e.target.value)}
                  placeholder={t('min_withdraw_placeholder', '最�?1000 积分起提')}
                  className="w-full pl-12 pr-4 py-4 bg-zinc-100 dark:bg-zinc-800 border-2 border-transparent focus:border-emerald-500 rounded-2xl font-bold text-xl text-zinc-900 dark:text-white outline-none transition-all"
                />
                <div className="absolute inset-y-0 right-0 pr-4 flex items-center">
                  <button 
                    onClick={() => setWithdrawAmount(profile?.points?.toString() || "0")}
                    className="text-sm font-bold text-emerald-500 hover:text-emerald-600"
                  >
                    {t('withdraw_all', '全部提现')}
                  </button>
                </div>
              </div>
              <p className="text-sm text-zinc-500 mt-2 flex justify-between">
                <span>{t('exchange_rate', '兑换比例: 100 积分 = ¥1')}</span>
                <span>{t('estimated_fiat', '预计到账')}: <strong className="text-emerald-500">¥{Math.floor((parseInt(withdrawAmount) || 0) / 100)}</strong></span>
              </p>
            </div>
            
            <div className="bg-amber-500/10 border border-amber-500/20 rounded-xl p-4 text-sm text-amber-600 dark:text-amber-400">
              <p className="font-bold mb-1">{t('withdraw_notice_title', '提现须知�?)}</p>
              <ul className="list-disc pl-5 space-y-1">
                <li>{t('withdraw_notice_1', '提现申请将在 24 小时内审核处理�?)}</li>
                <li>{t('withdraw_notice_2', '提现将收�?5% 的手续费�?)}</li>
                <li>{t('withdraw_notice_3', '请确保您已绑定有效的收款账户�?)}</li>
              </ul>
            </div>
          </motion.div>
        )}

        <div className="mt-10 flex justify-center">
          <button 
            disabled={(mode === "recharge" && !selectedAmount) || (mode === "withdraw" && (!withdrawAmount || parseInt(withdrawAmount) < 1000)) || isRecharging}
            onClick={handleRecharge}
            className={`flex items-center space-x-2 px-12 py-5 rounded-2xl font-black text-lg transition-all ${
              ((mode === "recharge" && selectedAmount) || (mode === "withdraw" && withdrawAmount && parseInt(withdrawAmount) >= 1000)) && !isRecharging
                ? mode === "recharge" 
                  ? "bg-blue-600 hover:bg-blue-500 text-white shadow-[0_0_30px_rgba(37,99,235,0.4)] hover:scale-105" 
                  : "bg-emerald-600 hover:bg-emerald-500 text-white shadow-[0_0_30px_rgba(16,185,129,0.4)] hover:scale-105"
                : "bg-zinc-200 dark:bg-zinc-800 text-zinc-400 cursor-not-allowed"
            }`}
          >
            {mode === "recharge" ? (
              <CreditCard size={24} fill="currentColor" className={isRecharging ? "animate-pulse" : ""} />
            ) : (
              <ArrowUpFromLine size={24} className={isRecharging ? "animate-pulse" : ""} />
            )}
            <span>{isRecharging ? t('processing', '处理�?..') : (mode === "recharge" ? t('pay_now', '立即支付') : t('confirm_withdraw', '确认提现'))}</span>
            <ArrowRight size={20} />
          </button>
        </div>
      </div>
    </div>
  );
}

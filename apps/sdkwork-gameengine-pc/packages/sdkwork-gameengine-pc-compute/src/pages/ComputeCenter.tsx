import React, { useState } from "react";
import { motion } from "motion/react";
import { Cpu, Zap, BatteryCharging, ArrowRight, Activity, Server, Database, CreditCard, Calculator } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUserStore } from "sdkwork-gameengine-pc-core";
import { useToast } from "sdkwork-gameengine-pc-commons";

export default function ComputeCenter() {
  const { t } = useTranslation();
  const { profile, deductPoints, addComputeTokens } = useUserStore();
  const { showToast, ToastComponent } = useToast();
  
  const [selectedPackage, setSelectedPackage] = useState<number | null>(2);
  const [customAmount, setCustomAmount] = useState<string>("");
  const [paymentMethod, setPaymentMethod] = useState<"points" | "fiat">("points");
  const [isProcessing, setIsProcessing] = useState(false);

  const computePackages = [
    { id: 1, tokens: 100000, label: "100K", popular: false, bonus: 0 },
    { id: 2, tokens: 500000, label: "500K", popular: true, bonus: 20000 },
    { id: 3, tokens: 1000000, label: "1M", popular: false, bonus: 100000 },
    { id: 4, tokens: 5000000, label: "5M", popular: false, bonus: 1000000 },
  ];

  const getSelectedTokens = () => {
    if (selectedPackage) {
      return computePackages.find(p => p.id === selectedPackage)?.tokens || 0;
    }
    return parseInt(customAmount) || 0;
  };

  const getBonusTokens = () => {
    if (selectedPackage) {
      return computePackages.find(p => p.id === selectedPackage)?.bonus || 0;
    }
    return 0;
  };

  const tokens = getSelectedTokens();
  const bonus = getBonusTokens();
  const totalTokens = tokens + bonus;
  
  // 100 Tokens = 1 Point = ¥0.01
  const pointsCost = Math.ceil(tokens / 100);
  const fiatCost = (tokens / 10000).toFixed(2);

  const handleCustomAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value.replace(/\D/g, '');
    setCustomAmount(val);
    if (val) {
      setSelectedPackage(null);
    } else {
      setSelectedPackage(1);
    }
  };

  const handlePackageSelect = (id: number) => {
    setSelectedPackage(id);
    setCustomAmount("");
  };

  const handleExchange = () => {
    if (tokens <= 0) return;

    if (paymentMethod === "points") {
      if (profile && profile.points >= pointsCost) {
        setIsProcessing(true);
        setTimeout(() => {
          const success = deductPoints(pointsCost);
          if (success) {
            addComputeTokens(totalTokens);
            showToast(t('exchange_success', '兑换成功�?), 'success');
          }
          setIsProcessing(false);
        }, 800);
      } else {
        showToast(t('insufficient_points', '积分不足'), 'error');
      }
    } else {
      setIsProcessing(true);
      setTimeout(() => {
        addComputeTokens(totalTokens);
        showToast(t('purchase_success', '购买成功�?), 'success');
        setIsProcessing(false);
      }, 800);
    }
  };

  return (
    <div className="space-y-8 pb-12 max-w-6xl mx-auto">
      <ToastComponent />
      {/* Header */}
      <div className="text-center space-y-4 py-12 relative">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-emerald-600/20 rounded-full blur-[100px] pointer-events-none"></div>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="relative z-10"
        >
          <div className="inline-flex items-center justify-center space-x-2 px-4 py-1.5 rounded-full bg-emerald-100 dark:bg-emerald-500/10 border border-emerald-200 dark:border-emerald-500/20 text-emerald-600 dark:text-emerald-400 text-sm font-bold mb-6">
            <Cpu size={16} />
            <span>{t('compute_center', '算力中心')}</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-black text-zinc-900 dark:text-white tracking-tight mb-4">
            {t('drive_your_agents', '驱动你的智能�?)}
          </h1>
          <p className="text-zinc-500 dark:text-zinc-400 text-lg max-w-2xl mx-auto font-medium">
            {t('compute_desc', '使用积分兑换或直接购买算�?(Token)，为你的龙虾 (Agent) 提供持续的思考能力�?)}
          </p>
        </motion.div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 relative z-10">
        <div className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm flex items-center space-x-4">
          <div className="w-14 h-14 bg-emerald-500/10 rounded-2xl flex items-center justify-center">
            <BatteryCharging className="text-emerald-500" size={28} />
          </div>
          <div>
            <p className="text-sm font-bold text-zinc-500 dark:text-zinc-400">{t('current_compute', '当前算力余额')}</p>
            <p className="text-3xl font-black text-zinc-900 dark:text-white">{profile?.computeTokens?.toLocaleString() || 0} <span className="text-sm text-zinc-500 font-medium">Tokens</span></p>
          </div>
        </div>
        
        <div className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm flex items-center space-x-4">
          <div className="w-14 h-14 bg-orange-500/10 rounded-2xl flex items-center justify-center">
            <Activity className="text-orange-500" size={28} />
          </div>
          <div>
            <p className="text-sm font-bold text-zinc-500 dark:text-zinc-400">{t('consumption_rate', '消耗速率')}</p>
            <p className="text-3xl font-black text-zinc-900 dark:text-white">1.2K <span className="text-sm text-zinc-500 font-medium">/ match</span></p>
          </div>
        </div>

        <div className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm flex items-center space-x-4">
          <div className="w-14 h-14 bg-rose-500/10 rounded-2xl flex items-center justify-center">
            <Database className="text-rose-500" size={28} />
          </div>
          <div>
            <p className="text-sm font-bold text-zinc-500 dark:text-zinc-400">{t('available_points', '可用积分')}</p>
            <p className="text-3xl font-black text-zinc-900 dark:text-white">{profile?.points?.toLocaleString() || 0} <span className="text-sm text-zinc-500 font-medium">pts</span></p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 relative z-10 mt-12">
        {/* Left Column: Package Selection */}
        <div className="lg:col-span-2 space-y-6">
          <h2 className="text-2xl font-black text-zinc-900 dark:text-white flex items-center">
            <Server className="mr-2 text-emerald-500" />
            {t('select_amount', '选择充值数�?)}
          </h2>
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {computePackages.map((pkg, i) => (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.1 }}
                key={pkg.id}
                onClick={() => handlePackageSelect(pkg.id)}
                className={`relative bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-4 rounded-2xl border-2 transition-all cursor-pointer overflow-hidden flex flex-col items-center justify-center text-center ${
                  selectedPackage === pkg.id 
                    ? "border-emerald-500 shadow-[0_0_20px_rgba(16,185,129,0.15)] bg-emerald-50/50 dark:bg-emerald-900/10" 
                    : "border-zinc-200 dark:border-zinc-800 hover:border-emerald-500/50"
                }`}
              >
                {pkg.popular && (
                  <div className="absolute top-0 right-0 bg-emerald-500 text-white text-[10px] font-black px-2 py-0.5 rounded-bl-lg uppercase tracking-wider">
                    {t('best_value', '最划算')}
                  </div>
                )}
                <h3 className="text-2xl font-black text-zinc-900 dark:text-white mt-2">{pkg.label}</h3>
                <p className="text-xs font-bold text-zinc-500 mb-2">Tokens</p>
                {pkg.bonus > 0 ? (
                  <div className="text-[10px] font-bold text-emerald-500 bg-emerald-500/10 px-2 py-1 rounded-full">
                    +{(pkg.bonus / 1000)}K {t('bonus', '赠�?)}
                  </div>
                ) : (
                  <div className="h-5"></div>
                )}
              </motion.div>
            ))}
          </div>

          <div className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
            <label className="flex items-center text-sm font-bold text-zinc-700 dark:text-zinc-300 mb-3">
              <Calculator size={16} className="mr-2 text-zinc-400" />
              {t('custom_amount', '自定义数�?)}
            </label>
            <div className="relative">
              <input
                type="text"
                value={customAmount}
                onChange={handleCustomAmountChange}
                placeholder={t('enter_custom_amount', '输入需要充值的 Token 数量')}
                className={`w-full bg-zinc-100 dark:bg-zinc-950 border-2 rounded-2xl px-4 py-4 text-lg font-bold text-zinc-900 dark:text-white outline-none transition-all ${
                  !selectedPackage && customAmount ? "border-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.1)]" : "border-transparent focus:border-zinc-300 dark:focus:border-zinc-700"
                }`}
              />
              <div className="absolute right-4 top-1/2 -translate-y-1/2 text-zinc-400 font-bold">
                Tokens
              </div>
            </div>
            <p className="text-xs text-zinc-500 mt-3 font-medium">
              {t('custom_amount_hint', '自定义数量不享受额外赠送，最�?100 Tokens 起充�?)}
            </p>
          </div>
        </div>

        {/* Right Column: Payment Method & Summary */}
        <div className="space-y-6">
          <h2 className="text-2xl font-black text-zinc-900 dark:text-white flex items-center">
            <CreditCard className="mr-2 text-emerald-500" />
            {t('payment_method', '支付方式')}
          </h2>

          <div className="bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm space-y-4">
            <button
              onClick={() => setPaymentMethod("points")}
              className={`w-full flex items-center justify-between p-4 rounded-2xl border-2 transition-all ${
                paymentMethod === "points"
                  ? "border-rose-500 bg-rose-50/50 dark:bg-rose-900/10"
                  : "border-zinc-200 dark:border-zinc-800 hover:border-rose-500/50"
              }`}
            >
              <div className="flex items-center space-x-3">
                <div className={`w-10 h-10 rounded-full flex items-center justify-center ${paymentMethod === "points" ? "bg-rose-500 text-white" : "bg-zinc-100 dark:bg-zinc-800 text-zinc-400"}`}>
                  <Database size={20} />
                </div>
                <div className="text-left">
                  <p className="font-bold text-zinc-900 dark:text-white">{t('pay_with_points', '积分支付')}</p>
                  <p className="text-xs text-zinc-500">{t('balance', '余额')}: {profile?.points?.toLocaleString()}</p>
                </div>
              </div>
              <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${paymentMethod === "points" ? "border-rose-500" : "border-zinc-300 dark:border-zinc-700"}`}>
                {paymentMethod === "points" && <div className="w-2.5 h-2.5 rounded-full bg-rose-500" />}
              </div>
            </button>

            <button
              onClick={() => setPaymentMethod("fiat")}
              className={`w-full flex items-center justify-between p-4 rounded-2xl border-2 transition-all ${
                paymentMethod === "fiat"
                  ? "border-blue-500 bg-blue-50/50 dark:bg-blue-900/10"
                  : "border-zinc-200 dark:border-zinc-800 hover:border-blue-500/50"
              }`}
            >
              <div className="flex items-center space-x-3">
                <div className={`w-10 h-10 rounded-full flex items-center justify-center ${paymentMethod === "fiat" ? "bg-blue-500 text-white" : "bg-zinc-100 dark:bg-zinc-800 text-zinc-400"}`}>
                  <CreditCard size={20} />
                </div>
                <div className="text-left">
                  <p className="font-bold text-zinc-900 dark:text-white">{t('direct_purchase', '直接购买')}</p>
                  <p className="text-xs text-zinc-500">{t('support_wechat_alipay', '支持微信/支付�?)}</p>
                </div>
              </div>
              <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${paymentMethod === "fiat" ? "border-blue-500" : "border-zinc-300 dark:border-zinc-700"}`}>
                {paymentMethod === "fiat" && <div className="w-2.5 h-2.5 rounded-full bg-blue-500" />}
              </div>
            </button>
          </div>

          <div className="bg-zinc-900 dark:bg-zinc-950 p-6 rounded-3xl border border-zinc-800 shadow-xl relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-emerald-500/10 rounded-full blur-2xl pointer-events-none"></div>
            
            <h3 className="text-lg font-bold text-white mb-4">{t('order_summary', '订单详情')}</h3>
            
            <div className="space-y-3 mb-6">
              <div className="flex justify-between text-sm">
                <span className="text-zinc-400">{t('base_tokens', '基础算力')}</span>
                <span className="font-bold text-white">{tokens.toLocaleString()}</span>
              </div>
              {bonus > 0 && (
                <div className="flex justify-between text-sm">
                  <span className="text-emerald-400">{t('bonus_tokens', '额外赠�?)}</span>
                  <span className="font-bold text-emerald-400">+{bonus.toLocaleString()}</span>
                </div>
              )}
              <div className="w-full h-px bg-zinc-800 my-2"></div>
              <div className="flex justify-between items-center">
                <span className="text-zinc-400 font-medium">{t('total_receive', '总计获得')}</span>
                <span className="text-2xl font-black text-emerald-400 flex items-center">
                  <Zap size={20} className="mr-1" />
                  {totalTokens.toLocaleString()}
                </span>
              </div>
            </div>

            <div className="bg-black/30 rounded-2xl p-4 mb-6">
              <div className="flex justify-between items-center">
                <span className="text-zinc-400 font-medium">{t('total_cost', '支付金额')}</span>
                {paymentMethod === "points" ? (
                  <span className="text-2xl font-black text-rose-500 flex items-center">
                    <Database size={20} className="mr-1" />
                    {pointsCost.toLocaleString()}
                  </span>
                ) : (
                  <span className="text-2xl font-black text-blue-400 flex items-center">
                    <span className="text-lg mr-1">¥</span>
                    {fiatCost}
                  </span>
                )}
              </div>
            </div>

            <button 
              disabled={tokens < 100 || isProcessing || (paymentMethod === "points" && profile && profile.points < pointsCost)}
              onClick={handleExchange}
              className={`w-full flex items-center justify-center space-x-2 py-4 rounded-2xl font-black text-lg transition-all ${
                tokens >= 100 && !isProcessing && (paymentMethod === "fiat" || (profile && profile.points >= pointsCost))
                  ? "bg-emerald-500 hover:bg-emerald-400 text-white shadow-[0_0_20px_rgba(16,185,129,0.3)] hover:scale-[1.02]" 
                  : "bg-zinc-800 text-zinc-500 cursor-not-allowed"
              }`}
            >
              {isProcessing ? (
                <span className="animate-pulse">{t('processing', '处理�?..')}</span>
              ) : (
                <>
                  <span>{paymentMethod === "points" ? t('confirm_exchange', '确认兑换') : t('confirm_purchase', '确认购买')}</span>
                  <ArrowRight size={20} />
                </>
              )}
            </button>
            
            {paymentMethod === "points" && profile && profile.points < pointsCost && tokens >= 100 && (
              <p className="text-rose-500 text-xs text-center mt-3 font-bold">
                {t('insufficient_points_hint', '积分余额不足，请调整数量或更换支付方�?)}
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

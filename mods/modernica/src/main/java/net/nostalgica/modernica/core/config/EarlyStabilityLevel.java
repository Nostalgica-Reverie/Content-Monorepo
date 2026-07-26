package net.nostalgica.modernica.core.config;

/** The stability level as seen by the early gating path, mirroring {@link ModernicaConfig.StabilityLevel}.
 *
 * <p>It exists so that path never <em>initializes</em> {@code ModernicaConfig.StabilityLevel}. That enum implements
 * fzzy-config's {@code EnumTranslatable}, and simply reading one of its constants initializes every
 * default-method superinterface with it (JLS 12.4.1): {@code EnumTranslatable} -> {@code Translatable}
 * -> {@code Translatable$Utils} -> {@code Translatable$Empty}, whose static initializer builds an empty
 * {@code Component}. That pulls {@code net.minecraft.network.chat.Style} and the rest of the component
 * classes into the class loader from inside {@link net.nostalgica.modernica.core.ModernicaMixinPlugin}'s
 * constructor - i.e. while Mixin is still selecting configs, so they load untransformed and permanently.
 * Any mod whose config is prepared after ours and mixes into one of them then dies with
 * "Critical problem: ... target net.minecraft.network.chat.Style was loaded too early."
 *
 * <p>Keep the constants identical to {@link ModernicaConfig.StabilityLevel} - {@link MixinGate#bindRealConfig}
 * checks that for us once the real config is safe to load. */
enum EarlyStabilityLevel {
    GA,
    BETA;

    boolean isAtLeast(EarlyStabilityLevel required) {
        return this.ordinal() >= required.ordinal();
    }
}

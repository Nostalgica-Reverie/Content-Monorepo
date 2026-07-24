plugins {
    id("net.fabricmc.fabric-loom") version "1.17-SNAPSHOT"
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

val buildId: String? = System.getenv("BUILD_ID")

base {
    archivesName.set("${sc.properties["mod.archives_base_name"] as String}-mc${sc.current.version}")
}
group = sc.properties["mod.group"] as String

version = buildString {
    append(sc.properties["mod.version"] as String)
    if (buildId != null) append("+build.$buildId")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(25))
    }
    withSourcesJar()
}

repositories {
    mavenCentral()
    maven("https://maven.fabricmc.net/") { name = "Fabric" }
    maven("https://api.modrinth.com/maven") { name = "Modrinth" }
    maven("https://maven.terraformersmc.com/releases/") { name = "TerraformersMC" }
    // velocity-native, for perf.network_optimizations (merged in from Krypton)
    maven("https://repo.papermc.io/repository/maven-public/") { name = "PaperMC" }
}

loom {
    accessWidenerPath.set(rootProject.file("src/main/resources/modernica.accesswidener"))
}

val mixinextras = "io.github.llamalad7:mixinextras-fabric:${sc.properties["deps.mixinextras"] as String}"

dependencies {
    minecraft("com.mojang:minecraft:${sc.current.version}")
    implementation("net.fabricmc:fabric-loader:${sc.properties["deps.fabric_loader"] as String}")
    implementation("net.fabricmc.fabric-api:fabric-api:${sc.properties["deps.fabric_api"] as String}")
    implementation("net.fabricmc:fabric-language-kotlin:${sc.properties["deps.fabric_language_kotlin"] as String}")

    // fzzy-config (Milestone 3): replaces ModernFix's ModernFixEarlyConfig/.properties system entirely.
    implementation("maven.modrinth:fzzy-config:${sc.properties["deps.fzzy_config"] as String}")

    // MixinExtras, needed by several ported Modernica mixins (@Local sugar etc.)
    implementation(include(mixinextras)!!)
    annotationProcessor(mixinextras)

    // Native AES cipher / native zlib compression for perf.network_optimizations (merged in from
    // Krypton, LGPL-3.0). Shaded in the same way Krypton itself shaded it.
    implementation(include("com.velocitypowered:velocity-native:3.4.0-SNAPSHOT")!!)

    // Optional integrations, kept from both source mods
    compileOnly("com.terraformersmc:modmenu:${sc.properties["deps.modmenu"] as String}") { isTransitive = false }
    compileOnly("maven.modrinth:spark:${sc.properties["deps.spark"] as String}")

    // compile-time only; fzzy-config ships this as a runtime jar-in-jar already
    compileOnly("net.peanuuutz.tomlkt:tomlkt-jvm:0.3.7")
}

tasks.processResources {
    inputs.property("version", version)
    inputs.property("minecraft", sc.current.version)

    filesMatching("fabric.mod.json") {
        expand("version" to version, "minecraft" to sc.current.version)
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(25)
    options.encoding = "UTF-8"
}

tasks.jar {
    from(rootProject.file("LICENSE"))
}

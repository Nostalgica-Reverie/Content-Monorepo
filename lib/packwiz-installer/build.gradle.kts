import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinVersion

plugins {
	java
	application
	id("com.gradleup.shadow") version "9.2.2"
	kotlin("jvm") version "2.2.20"
}

// Vendored into the Lasting Legacy monorepo from https://github.com/packwiz/packwiz-installer;
// release automation and git-derived versioning were removed in favour of a static version.
version = "1.0.0-packwand"

java {
	sourceCompatibility = JavaVersion.VERSION_1_8
	targetCompatibility = JavaVersion.VERSION_1_8
}

repositories {
	mavenCentral()
	google()
	maven {
		url = uri("https://jitpack.io")
	}
}

val r8 by configurations.creating

dependencies {
	implementation("commons-cli:commons-cli:1.5.0")
	implementation("com.google.code.gson:gson:2.9.0")
	implementation("com.squareup.okio:okio:3.1.0")
	implementation(kotlin("stdlib-jdk8"))
	implementation("com.squareup.okhttp3:okhttp:4.10.0")
	implementation("cc.ekblad:4koma:1.1.0")

	r8("com.android.tools:r8:8.5.35")
}

application {
	mainClass.set("link.infra.packwiz.installer.RequiresBootstrap")
}

tasks.jar {
	manifest {
		attributes["Main-Class"] = "link.infra.packwiz.installer.RequiresBootstrap"
		attributes["Implementation-Version"] = project.version
	}
}

tasks.shadowJar {
	// 4koma uses kotlin-reflect; requires Kotlin metadata
	exclude("META-INF/maven/**/*")
	exclude("META-INF/proguard/**/*")

	// Relocate Commons CLI, so that it doesn't clash with old packwiz-installer-bootstrap (that shades it)
	relocate("org.apache.commons.cli", "link.infra.packwiz.installer.deps.commons-cli")

	// from Commons CLI
	exclude("META-INF/LICENSE.txt")
	exclude("META-INF/NOTICE.txt")
}

val shrinkJar by tasks.registering(JavaExec::class) {
	val rules = file("src/main/proguard.txt")
	val r8File = base.libsDirectory.file(provider {
		base.archivesName.get() + "-" + project.version + "-all-shrink.jar"
	})
	dependsOn(configurations.named("runtimeClasspath"))
	inputs.files(tasks.shadowJar, rules)
	outputs.file(r8File)

	classpath(r8)
	mainClass.set("com.android.tools.r8.R8")
	args = mutableListOf(
		"--release",
		"--classfile",
		"--output", r8File.get().toString(),
		"--pg-conf", rules.toString(),
		"--lib", System.getProperty("java.home"),
		tasks.shadowJar.get().archiveFile.get().asFile.toString()
	)
}

// MANIFEST.MF must be one of the first 2 entries in the zip for JarInputStream to see it
// Gradle's JAR creation handles this whereas R8 doesn't, so the dist JAR is repacked
val distJar by tasks.registering(Jar::class) {
	from(shrinkJar.map { zipTree(it.outputs.files.singleFile) })
	archiveClassifier.set("all-repacked")
	manifest {
		from(shrinkJar.map { zipTree(it.outputs.files.singleFile).matching {
			include("META-INF/MANIFEST.MF")
		}.singleFile })
	}
}

// The default dist jar is the shadow jar. R8 shrinking (shrinkJar/distJar)
// is opt-in via -PshrinkDist=true: R8 8.5 cannot read the class files of very
// new JDKs (e.g. major version 69 / Java 25) passed as --lib.
val shrinkDist = providers.gradleProperty("shrinkDist").map { it.toBoolean() }.getOrElse(false)

val copyJar by tasks.registering(Copy::class) {
	if (shrinkDist) {
		from(distJar)
		rename("packwiz-installer-(.*)\\.jar", "packwiz-installer.jar")
	} else {
		from(tasks.shadowJar)
		rename("packwiz-installer-(.*)\\.jar", "packwiz-installer.jar")
	}
	into(layout.buildDirectory.dir("dist"))
	outputs.file(layout.buildDirectory.dir("dist").map { it.file("packwiz-installer.jar") })
}

tasks.build {
	dependsOn(copyJar)
}

kotlin {
	compilerOptions {
		jvmTarget.set(JvmTarget.JVM_1_8)
		languageVersion.set(KotlinVersion.KOTLIN_1_9)
		apiVersion.set(KotlinVersion.KOTLIN_1_9)
		freeCompilerArgs.addAll("-Xjvm-default=all", "-opt-in=kotlin.io.path.ExperimentalPathApi")
	}
}

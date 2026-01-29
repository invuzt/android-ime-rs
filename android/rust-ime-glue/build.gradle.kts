import com.android.build.gradle.BaseExtension
import org.gradle.kotlin.dsl.support.serviceOf
import java.util.zip.ZipFile

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "dev.matrix.rust.ime.glue"

    compileSdk {
        version = release(36)
    }

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
}

tasks.register("buildClassesDexDebug") {
    dependsOn("assembleDebug")

    group = "custom"
    description = "Extracts classes.jar from an AAR and converts it to a loadable Dex-JAR"

    val inputAarPath = layout.buildDirectory.file("outputs/aar/${project.name}-debug.aar").get().asFile
    val outputDir = layout.buildDirectory.dir("outputs/dex").get().asFile
    val classesJar = File(outputDir, "classes.jar")

    doLast {
        if (!outputDir.exists()) outputDir.mkdirs()

        // Extract classes.jar from AAR
        ZipFile(inputAarPath).use { zip ->
            val entry = zip.getEntry("classes.jar") ?: throw GradleException("No classes.jar in AAR")
            zip.getInputStream(entry).copyTo(classesJar.outputStream())
        }

        // Run D8 to convert JAR to DEX
        val baseExtension = project.extensions.getByType<BaseExtension>()
        val sdkDir = baseExtension.sdkDirectory
        val androidJar = File(sdkDir, "platforms/${baseExtension.compileSdkVersion}/android.jar")
        val buildToolsDir = File(sdkDir, "build-tools").listFiles().orEmpty().sortedArray().lastOrNull()
            ?: throw GradleException("Build tools not found")
        val d8Path = File(buildToolsDir, "d8").absolutePath

        project.serviceOf<ExecOperations>().exec {
            // it will output classes.dex
            commandLine(
                d8Path,
                "--output", outputDir.absolutePath,
                "--lib", androidJar.absolutePath,
                classesJar.absolutePath
            )
        }
    }
}

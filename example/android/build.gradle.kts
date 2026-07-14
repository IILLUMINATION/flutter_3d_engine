allprojects {
    repositories {
        google()
        mavenCentral()
    }
    afterEvaluate {
        if (hasProperty("android")) {
            val androidExt = extensions.findByName("android")
            if (androidExt is com.android.build.api.dsl.LibraryExtension || androidExt is com.android.build.api.dsl.CommonExtension<*, *, *, *>) {
                try {
                    if (androidExt.compileSdk == null || (androidExt.compileSdk as? Int ?: 0) < 34) {
                        println("Forcing compileSdk to 35 for ${project.name}")
                        androidExt.compileSdk = 35
                    }
                } catch (_: Exception) {}
            }
        }
    }
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)
}
subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}

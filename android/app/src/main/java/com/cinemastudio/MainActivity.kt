package com.cinemastudio

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.cinemastudio.engine.EngineBridge
import com.cinemastudio.ui.EditorScreen
import com.cinemastudio.ui.HomeScreen
import com.cinemastudio.ui.SettingsScreen
import com.cinemastudio.ui.theme.CinemaStudioTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        EngineBridge.initialize()

        setContent {
            CinemaStudioTheme {
                Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
                    val nav = rememberNavController()
                    NavHost(navController = nav, startDestination = "home") {
                        composable("home") {
                            HomeScreen(
                                onOpenEditor = { nav.navigate("editor") },
                                onOpenSettings = { nav.navigate("settings") }
                            )
                        }
                        composable("editor") {
                            EditorScreen(onBack = { nav.popBackStack() })
                        }
                        composable("settings") {
                            SettingsScreen(onBack = { nav.popBackStack() })
                        }
                    }
                }
            }
        }
    }
}

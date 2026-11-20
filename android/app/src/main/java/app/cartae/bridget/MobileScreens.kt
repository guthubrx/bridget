package app.cartae.bridget

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * Composants d'écran modulaires extraits de `MainActivity.kt` (spec 306) :
 * recherche dans l'historique mobile, filtres de notifications et actions opt-in d'agent.
 */
@Composable
fun LedgerSearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    onSubmit: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth().padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        OutlinedTextField(
            value = query,
            onValueChange = onQueryChange,
            label = { Text("Rechercher dans l'historique") },
            singleLine = true,
            modifier = Modifier.weight(1f),
        )
        Button(onClick = onSubmit) {
            Text("Chercher")
        }
    }
}

@Composable
fun NotificationFilterCard(
    preferences: NotificationFilterPreferences,
    availableProjects: List<String>,
    onUrgencyChange: (NotificationUrgency) -> Unit,
    onToggleProject: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(modifier = modifier.fillMaxWidth().padding(vertical = 4.dp)) {
        Column(modifier = Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Filtres de notifications", style = MaterialTheme.typography.titleSmall)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                NotificationUrgency.entries.forEach { level ->
                    FilterChip(
                        selected = preferences.minUrgency == level,
                        onClick = { onUrgencyChange(level) },
                        label = { Text(level.wire) },
                    )
                }
            }
            if (availableProjects.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    availableProjects.take(6).forEach { project ->
                        FilterChip(
                            selected = project in preferences.projects,
                            onClick = { onToggleProject(project) },
                            label = { Text(project) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun OptInAgentActionsRow(
    agent: MobileAgentSummary,
    optInEnabled: Boolean,
    onRelaunch: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!optInEnabled || !agent.canRelaunch) return
    Row(modifier = modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
        Button(onClick = { onRelaunch(agent.id) }) {
            Text("Relancer ${agent.name.ifBlank { agent.id }}")
        }
    }
}

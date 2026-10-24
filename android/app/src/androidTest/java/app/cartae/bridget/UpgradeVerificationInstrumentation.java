package app.cartae.bridget;

import android.app.Activity;
import android.app.Instrumentation;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.os.Bundle;
import java.nio.charset.StandardCharsets;
import java.security.KeyStore;
import java.util.Arrays;
import javax.crypto.Cipher;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

/** Vérifie les données antérieures sans dépendre des classes optimisées de l'application. */
public final class UpgradeVerificationInstrumentation extends Instrumentation {
    @Override public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        start();
    }
    @Override public void onStart() {
        Bundle result = new Bundle();
        try {
            byte[] encrypted;
            try (SQLiteDatabase db = SQLiteDatabase.openDatabase(
                    getTargetContext().getDatabasePath("upgrade-mobile177.db").getAbsolutePath(),
                    null, SQLiteDatabase.OPEN_READONLY);
                 Cursor row = db.rawQuery("SELECT value FROM entries WHERE id=?", new String[]{"proof"})) {
                if (!row.moveToFirst()) throw new IllegalStateException();
                encrypted = row.getBlob(0);
            }
            KeyStore keys = KeyStore.getInstance("AndroidKeyStore");
            keys.load(null);
            SecretKey key = (SecretKey) keys.getKey("bridget.mobile177.upgrade-proof", null);
            Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
            cipher.init(Cipher.DECRYPT_MODE, key, new GCMParameterSpec(128, Arrays.copyOfRange(encrypted, 0, 12)));
            cipher.updateAAD("proof".getBytes(StandardCharsets.UTF_8));
            String plain = new String(cipher.doFinal(Arrays.copyOfRange(encrypted, 12, encrypted.length)), StandardCharsets.UTF_8);
            if (!plain.equals("{\"value\":\"preserve177\"}")) throw new IllegalStateException();
            getTargetContext().deleteDatabase("upgrade-mobile177.db");
            keys.deleteEntry("bridget.mobile177.upgrade-proof");
            result.putString("upgrade_storage", "preserved_after_certificate_rotation");
            finish(Activity.RESULT_OK, result);
        } catch (Exception error) {
            result.putString("upgrade_storage", "unconfirmed");
            finish(Activity.RESULT_CANCELED, result);
        }
    }
}

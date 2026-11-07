package com.example.app;

import android.app.Activity;
import android.os.Bundle;

public class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Reference some strings
        String title = getString(R.string.bold_text);
        String message = getString(R.string.email_template);
        String path = getString(R.string.file_path);
    }
}


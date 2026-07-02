package link.infra.packwiz.installer.bootstrap;

import javax.swing.*;
import java.awt.*;
import java.io.IOException;
import java.io.InputStream;
import java.io.InterruptedIOException;
import java.net.URLConnection;
import java.util.concurrent.atomic.AtomicBoolean;

class ConnMonitorInputStream extends InputStream {
	private InputStream in = null;
	private int size = -1;
	private int bytesRead = 0;
	private final URLConnection conn;
	private ProgressMonitor mon;

	public ConnMonitorInputStream(URLConnection conn, String message, String note) {
		this.conn = conn;
		EventQueue.invokeLater(() -> {
			mon = new ProgressMonitor(null, message, note, 0, 1);
			mon.setMillisToDecideToPopup(1);
			mon.setMillisToPopup(1);
		});
	}

	private void setup() throws IOException {
		if (in == null) {
			try {
				size = conn.getContentLength();
				in = conn.getInputStream();
				EventQueue.invokeLater(() -> {
					mon.setProgress(0);
					if (size > -1) {
						mon.setMaximum(size);
					}
				});
			} catch (IOException e) {
				EventQueue.invokeLater(() -> mon.close());
				throw e;
			}
		}
	}

	public int available() {
		if (size > -1) {
			return (size - bytesRead);
		} else {
			return 1;
		}
	}

	private final AtomicBoolean wasCancelled = new AtomicBoolean();
	private long lastMillisUpdated = System.currentTimeMillis() - 110;

	private void setProgress() throws InterruptedIOException {
		// Update at most once every 100 ms
		if (System.currentTimeMillis() - lastMillisUpdated < 100) {
			return;
		}
		lastMillisUpdated = System.currentTimeMillis();
		final int progress = size > -1 ? bytesRead : -1;
		EventQueue.invokeLater(() -> {
			if (progress > -1) {
				mon.setProgress(progress);
			}
			wasCancelled.set(mon.isCanceled());
		});
		if (wasCancelled.get()) {
			throw new InterruptedIOException("Download cancelled!");
		}
	}

	public int read() throws IOException {
		setup();
		int b = in.read();
		if (b != -1) {
			bytesRead++;
			setProgress();
		}
		return b;
	}

	public int read(byte[] b) throws IOException {
		setup();
		int read = in.read(b);
		bytesRead += read;
		setProgress();
		return read;
	}

	public int read(byte[] b, int off, int len) throws IOException {
		setup();
		int read = in.read(b, off, len);
		bytesRead += read;
		setProgress();
		return read;
	}

	public void close() throws IOException {
		super.close();
		EventQueue.invokeLater(() -> mon.close());
		if (wasCancelled.get()) {
			throw new InterruptedIOException("Download cancelled!");
		}
	}
}

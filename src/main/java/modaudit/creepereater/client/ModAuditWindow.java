package modaudit.creepereater.client;

import javax.swing.*;
import java.awt.*;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

public final class ModAuditWindow {
	private ModAuditWindow() {}

	public static void open(List<ExternalMod> missingMods, Path modsDirectory) {
		applySystemStyle();

		JFrame frame = new JFrame("Mod Audit");
		frame.setDefaultCloseOperation(WindowConstants.DISPOSE_ON_CLOSE);
		frame.setLayout(new BorderLayout(12, 12));
		frame.add(createHeader(), BorderLayout.NORTH);
		frame.add(createModList(frame, missingMods, modsDirectory), BorderLayout.CENTER);
		frame.add(createFooter(frame, modsDirectory), BorderLayout.SOUTH);
		frame.getRootPane().setBorder(BorderFactory.createEmptyBorder(16, 16, 16, 16));
		frame.setMinimumSize(new Dimension(560, 380));
		frame.pack();
		frame.setLocationRelativeTo(null);
		frame.setAlwaysOnTop(true);
		frame.setVisible(true);
		frame.toFront();
		frame.requestFocus();

		Timer timer = new Timer(1500, event -> frame.setAlwaysOnTop(false));
		timer.setRepeats(false);
		timer.start();
	}

	private static JPanel createHeader() {
		JPanel header = new JPanel();
		header.setLayout(new BoxLayout(header, BoxLayout.Y_AXIS));

		JLabel title = new JLabel("Additional Mods Available", SwingConstants.CENTER);
		title.setFont(title.getFont().deriveFont(Font.BOLD, 18f));
		title.setAlignmentX(JLabel.CENTER_ALIGNMENT);

		JLabel message = new JLabel(
			"<html><div style='text-align:center'>These externally distributed mods are not installed.<br>Download them, place them in your mods folder, and restart Minecraft.</div></html>",
			SwingConstants.CENTER
		);
		message.setAlignmentX(JLabel.CENTER_ALIGNMENT);

		header.add(title);
		header.add(Box.createVerticalStrut(8));
		header.add(message);
		return header;
	}

	private static JScrollPane createModList(JFrame frame, List<ExternalMod> missingMods, Path modsDirectory) {
		JPanel list = new JPanel();
		list.setLayout(new BoxLayout(list, BoxLayout.Y_AXIS));

		for (ExternalMod mod : missingMods) {
			JPanel row = new JPanel(new BorderLayout(12, 0));
			row.setBorder(BorderFactory.createEmptyBorder(6, 6, 6, 6));
			row.setMaximumSize(new Dimension(Integer.MAX_VALUE, 40));
			row.setAlignmentX(JPanel.LEFT_ALIGNMENT);
			JLabel name = new JLabel(mod.name());
			row.add(name, BorderLayout.CENTER);

			JButton download = new JButton("Download");
			download.addActionListener(event -> download(frame, mod, name, download, modsDirectory));
			row.add(download, BorderLayout.EAST);
			list.add(row);
		}

		JScrollPane scrollPane = new JScrollPane(list);
		scrollPane.setPreferredSize(new Dimension(520, 220));
		return scrollPane;
	}

	private static JPanel createFooter(JFrame frame, Path modsDirectory) {
		JPanel footer = new JPanel(new FlowLayout(FlowLayout.RIGHT));

		JButton openFolder = new JButton("Open Mods Folder");
		openFolder.addActionListener(event -> openModsFolder(frame, modsDirectory));
		footer.add(openFolder);

		JButton continueButton = new JButton("Continue");
		continueButton.addActionListener(event -> frame.dispose());
		footer.add(continueButton);
		return footer;
	}

	private static void applySystemStyle() {
		try {
			UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName());
		} catch (Exception exception) {
			ModAudit.LOGGER.warn("Failed to apply the system look and feel", exception);
		}
	}

	private static void download(JFrame frame, ExternalMod mod, JLabel name, JButton button, Path modsDirectory) {
		button.setEnabled(false);
		button.setText("Downloading...");
		ModDownloadService.download(mod, modsDirectory).whenComplete((path, failure) -> SwingUtilities.invokeLater(() -> {
			if (failure == null) {
				button.setText("Downloaded");
				name.setText(mod.name() + " — Restart Minecraft to load it");
				return;
			}

			button.setEnabled(true);
			button.setText("Retry");
			Throwable cause = failure.getCause() == null ? failure : failure.getCause();
			showError(frame, "Could not download " + mod.name() + ".\n" + cause.getMessage(), cause);
		}));
	}

	private static void openModsFolder(JFrame frame, Path modsDirectory) {
		try {
			if (!Desktop.isDesktopSupported()) {
				throw new IOException("Desktop integration is unavailable");
			}
			Files.createDirectories(modsDirectory);
			Desktop.getDesktop().open(modsDirectory.toFile());
		} catch (IOException exception) {
			showError(frame, "Could not open the mods folder.", exception);
		}
	}

	private static void showError(JFrame frame, String message, Throwable exception) {
		ModAudit.LOGGER.error(message, exception);
		JOptionPane.showMessageDialog(frame, message, "Mod Audit", JOptionPane.ERROR_MESSAGE);
	}
}

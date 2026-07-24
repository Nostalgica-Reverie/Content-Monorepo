package net.nostalgica.modernica.resources;

import it.unimi.dsi.fastutil.ints.IntArrayList;
import it.unimi.dsi.fastutil.ints.IntList;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.PackResources;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.resources.IoSupplier;

import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.channels.FileChannel;
import java.nio.channels.SeekableByteChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HashSet;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

/**
 * Indexes a zip central directory so namespace/resource listing can avoid full zip scans.
 */
public class ZipPackIndex {
    private static final int EOCD_SIGNATURE = 0x06054b50;
    private static final int EOCD_SIZE = 22;
    private static final int EOCD_OFF_CD_SIZE = 12;
    private static final int EOCD_OFF_CD_OFFSET = 16;
    private static final int EOCD_MAX_COMMENT_LENGTH = 65535;

    private static final int CD_ENTRY_SIGNATURE = 0x02014b50;
    private static final int CD_ENTRY_HEADER_SIZE = 46;
    private static final int CD_OFF_FILENAME_LENGTH = 28;
    private static final int CD_OFF_EXTRA_LENGTH = 30;
    private static final int CD_OFF_COMMENT_LENGTH = 32;

    private static final IntList EMPTY_OFFSETS = IntList.of();

    static final class DirNode {
        Map<String, DirNode> childDirs;
        IntList fileChildOffsets;

        DirNode() {
            childDirs = new Object2ObjectOpenHashMap<>();
            fileChildOffsets = EMPTY_OFFSETS;
        }

        void freeze() {
            if (fileChildOffsets instanceof IntArrayList arrayList) {
                arrayList.trim();
            }
            childDirs = childDirs.isEmpty() ? Map.of() : Map.copyOf(childDirs);
            for (DirNode child : childDirs.values()) {
                child.freeze();
            }
        }
    }

    private final ByteBuffer cdBuffer;
    private final Set<String> trackedTopLevelDirs;
    private final DirNode root;

    public ZipPackIndex(Path zipPath) throws IOException {
        this.cdBuffer = readCentralDirectory(zipPath);
        Set<String> packTypeDirs = new HashSet<>();
        for (PackType type : PackType.values()) {
            packTypeDirs.add(type.getDirectory());
        }
        this.trackedTopLevelDirs = Set.copyOf(packTypeDirs);
        this.root = buildTree();
    }

    private static SeekableByteChannel obtainChannel(Path filePath) throws IOException {
        try {
            return FileChannel.open(filePath, StandardOpenOption.READ);
        } catch (Exception e) {
            return Files.newByteChannel(filePath);
        }
    }

    private static ByteBuffer readCentralDirectory(Path filePath) throws IOException {
        try (SeekableByteChannel channel = obtainChannel(filePath)) {
            long fileSize = channel.size();
            if (fileSize < EOCD_SIZE) return null;

            int tailSize = (int)Math.min(fileSize, (long)EOCD_SIZE + EOCD_MAX_COMMENT_LENGTH);
            ByteBuffer tail = ByteBuffer.allocate(tailSize);
            tail.order(ByteOrder.LITTLE_ENDIAN);

            long tailStart = fileSize - tailSize;
            while (tail.hasRemaining()) {
                channel.position(tailStart + tail.position());
                int n = channel.read(tail);
                if (n < 0) {
                    break;
                }
            }
            if (tail.hasRemaining()) {
                throw new IOException("Failed to read ZIP tail");
            }
            tail.flip();

            int eocdPos = -1;
            for (int i = tailSize - EOCD_SIZE; i >= 0; i--) {
                if (tail.getInt(i) == EOCD_SIGNATURE) {
                    int commentLen = Short.toUnsignedInt(tail.getShort(i + 20));
                    if (i + EOCD_SIZE + commentLen == tailSize) {
                        eocdPos = i;
                        break;
                    }
                }
            }
            if (eocdPos < 0) return null;

            long cdSize = Integer.toUnsignedLong(tail.getInt(eocdPos + EOCD_OFF_CD_SIZE));
            long cdOffset = Integer.toUnsignedLong(tail.getInt(eocdPos + EOCD_OFF_CD_OFFSET));
            if (cdSize == 0) return null;
            if (cdSize == 0xFFFFFFFFL || cdOffset == 0xFFFFFFFFL) {
                throw new IOException("ZIP64 not supported by ZipPackIndex");
            }
            if (cdOffset > fileSize - cdSize) {
                throw new IOException("Invalid central directory range");
            }

            if (channel instanceof FileChannel fc) {
                try {
                    ByteBuffer buf = fc.map(FileChannel.MapMode.READ_ONLY, cdOffset, cdSize);
                    buf.order(ByteOrder.LITTLE_ENDIAN);
                    return buf;
                } catch (Exception ignored) {
                }
            }

            ByteBuffer buf = ByteBuffer.allocate((int)cdSize);
            buf.order(ByteOrder.LITTLE_ENDIAN);
            while (buf.hasRemaining()) {
                channel.position(cdOffset + buf.position());
                int n = channel.read(buf);
                if (n < 0) throw new IOException("Truncated central directory during heap read");
            }
            buf.flip();
            return buf;
        }
    }

    private DirNode buildTree() throws IOException {
        DirNode treeRoot = new DirNode();
        if (cdBuffer == null) {
            treeRoot.freeze();
            return treeRoot;
        }

        int pos = 0;
        int limit = cdBuffer.limit();
        while (pos + CD_ENTRY_HEADER_SIZE <= limit) {
            if (cdBuffer.getInt(pos) != CD_ENTRY_SIGNATURE) break;
            pos += indexCdEntry(pos, limit, treeRoot, cdBuffer);
        }

        treeRoot.freeze();
        return treeRoot;
    }

    private int indexCdEntry(int pos, int limit, DirNode treeRoot, ByteBuffer cdBuffer) throws IOException {
        int fileNameLen = Short.toUnsignedInt(cdBuffer.getShort(pos + CD_OFF_FILENAME_LENGTH));
        int extraLen = Short.toUnsignedInt(cdBuffer.getShort(pos + CD_OFF_EXTRA_LENGTH));
        int commentLen = Short.toUnsignedInt(cdBuffer.getShort(pos + CD_OFF_COMMENT_LENGTH));
        int recordLen = CD_ENTRY_HEADER_SIZE + fileNameLen + extraLen + commentLen;
        if (pos + recordLen > limit) {
            throw new IOException("Truncated central directory");
        }

        byte[] nameBytes = new byte[fileNameLen];
        cdBuffer.get(pos + CD_ENTRY_HEADER_SIZE, nameBytes);

        DirNode current = treeRoot;
        boolean tracked = false;
        boolean skipped = false;
        int segStart = 0;

        for (int i = 0; i < fileNameLen; i++) {
            if (nameBytes[i] == '/') {
                int segLen = i - segStart;
                if (segLen > 0) {
                    String segment = new String(nameBytes, segStart, segLen, StandardCharsets.UTF_8);
                    if (!tracked) {
                        if (!trackedTopLevelDirs.contains(segment)) {
                            skipped = true;
                            break;
                        }
                        tracked = true;
                    }
                    DirNode next = current.childDirs.get(segment);
                    if (next == null) {
                        current.childDirs.put(segment, next = new DirNode());
                    }
                    current = next;
                }
                segStart = i + 1;
            }
        }

        if (!skipped && tracked && segStart < fileNameLen) {
            if (current.fileChildOffsets == EMPTY_OFFSETS) {
                current.fileChildOffsets = new IntArrayList();
            }
            current.fileChildOffsets.add(pos);
        }

        return recordLen;
    }

    String readBasename(int cdOffset) {
        int nameLen = Short.toUnsignedInt(cdBuffer.getShort(cdOffset + CD_OFF_FILENAME_LENGTH));
        byte[] nameBytes = new byte[nameLen];
        cdBuffer.get(cdOffset + CD_ENTRY_HEADER_SIZE, nameBytes);
        int lastSlash = -1;
        for (int i = nameBytes.length - 1; i >= 0; i--) {
            if (nameBytes[i] == '/') {
                lastSlash = i;
                break;
            }
        }
        return new String(nameBytes, lastSlash + 1, nameLen - lastSlash - 1, StandardCharsets.UTF_8);
    }

    public Set<String> getTrackedTopLevelDirs() {
        return this.trackedTopLevelDirs;
    }

    public Set<String> getNamespaces(PackType type) {
        DirNode typeNode = root.childDirs.get(type.getDirectory());
        if (typeNode == null) return Set.of();
        Set<String> result = new HashSet<>();
        for (String ns : typeNode.childDirs.keySet()) {
            if (ns.equals(ns.toLowerCase(Locale.ROOT))) {
                result.add(ns);
            }
        }
        return result;
    }

    public boolean hasResource(String... paths) {
        var node = this.root;
        for (int i = 0; i < paths.length - 1; i++) {
            var path = paths[i];
            if (path.isEmpty()) {
                continue;
            }
            node = node.childDirs.get(path);
            if (node == null) {
                return false;
            }
        }
        String basename = paths[paths.length - 1];
        var offsets = node.fileChildOffsets;
        for (int i = 0; i < offsets.size(); i++) {
            if (basename.equals(readBasename(offsets.getInt(i)))) {
                return true;
            }
        }
        return false;
    }

    public void listResources(PackType type, String namespace, String path,
                              ZipFile zipFile, PackResources.ResourceOutput output) {
        DirNode node = root.childDirs.get(type.getDirectory());
        if (node == null) return;
        node = node.childDirs.get(namespace);
        if (node == null) return;

        String rlSubPath;
        if (!path.isEmpty()) {
            for (String segment : path.split("/")) {
                if (segment.isEmpty()) continue;
                node = node.childDirs.get(segment);
                if (node == null) return;
            }
            rlSubPath = path + "/";
        } else {
            rlSubPath = "";
        }

        String entryPrefix = type.getDirectory() + "/" + namespace + "/";
        collectResources(node, entryPrefix, rlSubPath, zipFile, namespace, output);
    }

    private void collectResources(DirNode node, String entryPrefix, String rlSubPath,
                                  ZipFile zipFile, String namespace,
                                  PackResources.ResourceOutput output) {
        var offsets = node.fileChildOffsets;
        for (int i = 0; i < offsets.size(); i++) {
            String basename = readBasename(offsets.getInt(i));
            String rlPathFull = rlSubPath + basename;
            Identifier rl = Identifier.tryBuild(namespace, rlPathFull);
            if (rl != null) {
                ZipEntry entry = zipFile.getEntry(entryPrefix + rlPathFull);
                if (entry != null) {
                    output.accept(rl, IoSupplier.create(zipFile, entry));
                }
            }
        }
        for (Map.Entry<String, DirNode> child : node.childDirs.entrySet()) {
            collectResources(child.getValue(), entryPrefix,
                    rlSubPath + child.getKey() + "/", zipFile, namespace, output);
        }
    }
}

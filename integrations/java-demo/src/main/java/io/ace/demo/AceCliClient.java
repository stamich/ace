package io.ace.demo;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

/**
 * Minimal process-based Java demonstration client for ACE 0.1.
 *
 * <p>The class deliberately does not expose JNI or Panama. It demonstrates how a JVM application can use the
 * milestone-0.1 CLI while keeping the Rust compression core independent of Java.</p>
 */
public final class AceCliClient {
    private final Path executable;

    /**
     * Creates a client pointing at an already-built ACE executable.
     *
     * @param executable path to the {@code ace} binary
     */
    public AceCliClient(Path executable) { this.executable = executable; }

    /**
     * Compresses one input file to an ACE container.
     *
     * @param input source file
     * @param output destination ACE file
     * @return child-process exit code
     * @throws IOException if the process cannot be started
     * @throws InterruptedException if the calling thread is interrupted while waiting
     */
    public int compress(Path input, Path output) throws IOException, InterruptedException {
        return run(List.of("compress", input.toString(), output.toString()));
    }

    /**
     * Decompresses one ACE container to a regular file.
     *
     * @param input source ACE file
     * @param output reconstructed destination file
     * @return child-process exit code
     * @throws IOException if the process cannot be started
     * @throws InterruptedException if the calling thread is interrupted while waiting
     */
    public int decompress(Path input, Path output) throws IOException, InterruptedException {
        return run(List.of("decompress", input.toString(), output.toString()));
    }

    /**
     * Executes one ACE CLI command and inherits standard input/output/error from the current JVM.
     *
     * @param arguments arguments following the executable name
     * @return child-process exit code
     * @throws IOException if the process cannot be started
     * @throws InterruptedException if the calling thread is interrupted while waiting
     */
    private int run(List<String> arguments) throws IOException, InterruptedException {
        var command = new java.util.ArrayList<String>();
        command.add(executable.toString()); command.addAll(arguments);
        return new ProcessBuilder(command).inheritIO().start().waitFor();
    }
}

package io.ace.demo

import java.nio.file.Path
import scala.sys.process.Process

/**
 * Minimal process-based Scala demonstration client for ACE 0.1.
 *
 * The wrapper keeps Scala outside the compression core while showing how a JVM/Scala service can invoke the
 * milestone CLI. Native FFM bindings are intentionally deferred to a later milestone.
 *
 * @param executable path to the already-built `ace` executable
 */
final class AceCliClient(executable: Path) {

  /** Compresses a source file into an ACE container and returns the child-process exit code.
    *
    * @param input source file
    * @param output destination ACE file
    * @return process exit status
    */
  def compress(input: Path, output: Path): Int =
    run(Seq("compress", input.toString, output.toString))

  /** Decompresses an ACE container and returns the child-process exit code.
    *
    * @param input source ACE file
    * @param output reconstructed destination file
    * @return process exit status
    */
  def decompress(input: Path, output: Path): Int =
    run(Seq("decompress", input.toString, output.toString))

  /** Runs one ACE CLI command synchronously.
    *
    * @param args arguments following the executable name
    * @return process exit status
    */
  private def run(args: Seq[String]): Int =
    Process(executable.toString +: args).!
}

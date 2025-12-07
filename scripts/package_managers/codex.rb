# Homebrew Formula for Codex AI
# This formula allows installation of Codex via Homebrew

class Codex < Formula
  desc "CLI-based AI programming tool with local knowledge base and multi-AI platform support"
  homepage "https://github.com/lyxamour/codex"
  url "https://github.com/lyxamour/codex/archive/refs/tags/v0.4.4.tar.gz"
  sha256 "TODO: Generate with `shasum -a 256 <tarball>`"
  license "MIT"
  head "https://github.com/lyxamour/codex.git", branch: "main"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build

  def install
    # Build with cargo
    system "cargo", "install", *std_cargo_args

    # Create configuration directories
    (etc/"codex").mkpath
    (var/"codex").mkpath
    (var/"codex"/"logs").mkpath
    (var/"codex"/"data").mkpath
    (var/"codex"/"cache").mkpath
  end

  test do
    # Basic test to check if Codex runs
    system bin/"codex", "--version"
    
    # Test that help works
    system bin/"codex", "--help"
    
    # Test that chat command doesn't crash
    assert_match "Codex", shell_output("#{bin}/codex chat --help")
  end

  service do
    run [opt_bin/"codex", "--daemon"]
    keep_alive true
    working_dir var/"codex"
    log_path var/"log/codex.log"
    error_log_path var/"log/codex.error.log"
  end

  def caveats
    <<~EOS
      Codex AI has been installed!
      
      Configuration files are located in #{etc}/codex/
      Data files are located in #{var}/codex/
      
      To get started:
        1. Run `codex --help` to see available commands
        2. Run `codex chat` to start the interactive chat interface
        3. Index your codebase with `codex index /path/to/your/code`
        4. Configure AI providers in #{etc}/codex/ai.yaml
      
      For more information, visit #{homepage}
    EOS
  end
end

package cmd

import (
	"fmt"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/pflag"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// Command group IDs â€” used by subpackages to slot commands into the right section.
const (
	GroupPackManagement = "pack"
	GroupUpdates        = "update"
	GroupBuildExport    = "build"
	GroupWorkspace      = "workspace"
	GroupInfo           = "info"
	GroupOther          = "other"
)

var packFile string
var cfgFile string

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "packwand",
	Short: "Minecraft modpack toolchain â€” packwiz core with multi-pack workspace management",
	// The branded status bar is printed once per invocation (to stderr, and
	// only on interactive terminals) before any subcommand runs.
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		if cmd.Name() == "help" || cmd.Name() == "completion" {
			return
		}
		context := cmd.CommandPath()
		if cwd, err := os.Getwd(); err == nil {
			context += "  " + filepath.Base(cwd)
		}
		StatusBar(context)
	},
	Run: func(cmd *cobra.Command, args []string) {
		printMascot()
		_ = cmd.Help()
	},
}

var versionCmd = &cobra.Command{
	Use:     "version",
	Short:   "Print the packwand version",
	GroupID: GroupInfo,
	Run: func(cmd *cobra.Command, args []string) {
		printMascot()
		fmt.Println("packwand " + packwandVersion)
	},
}

// packwandVersion is the fallback for source builds; release builds override it
// via -ldflags "-X .../cmd.packwandVersion=<version>" (see .goreleaser.yml).
var packwandVersion = "26.2.0"

func Version() string {
	return packwandVersion
}

// CommandInfo is the stable, read-only command metadata exposed to local tools
// such as the GUI. It is generated from Cobra's registered command tree so
// consumers do not need to maintain a second list of Packwand features.
type CommandInfo struct {
	Path     string `json:"path"`
	Use      string `json:"use"`
	Summary  string `json:"summary"`
	Group    string `json:"group,omitempty"`
	Runnable bool   `json:"runnable"`
}

// CommandCatalog returns all visible Packwand commands except the root.
func CommandCatalog() []CommandInfo {
	commands := make([]CommandInfo, 0)
	var visit func(*cobra.Command)
	visit = func(parent *cobra.Command) {
		for _, command := range parent.Commands() {
			if command.Hidden {
				continue
			}
			path := strings.TrimPrefix(command.CommandPath(), rootCmd.Name()+" ")
			commands = append(commands, CommandInfo{
				Path:     path,
				Use:      command.Use,
				Summary:  command.Short,
				Group:    command.GroupID,
				Runnable: command.Runnable(),
			})
			visit(command)
		}
	}
	visit(rootCmd)
	return commands
}

// Execute starts the root command for packwand
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}

// Add adds a new command as a subcommand to packwand (no group).
func Add(newCommand *cobra.Command) {
	rootCmd.AddCommand(newCommand)
}

// AddToGroup adds a new command to rootCmd under the given group.
func AddToGroup(newCommand *cobra.Command, groupID string) {
	newCommand.GroupID = groupID
	rootCmd.AddCommand(newCommand)
}

func init() {
	cobra.OnInitialize(initConfig)

	// Register command groups so they appear as sections in --help output.
	rootCmd.AddGroup(
		&cobra.Group{ID: GroupPackManagement, Title: "Pack Management:"},
		&cobra.Group{ID: GroupUpdates, Title: "Updates & Refresh:"},
		&cobra.Group{ID: GroupBuildExport, Title: "Build & Export:"},
		&cobra.Group{ID: GroupWorkspace, Title: "Workspace (multi-pack):"},
		&cobra.Group{ID: GroupInfo, Title: "Information & Diagnostics:"},
		&cobra.Group{ID: GroupOther, Title: "Other:"},
	)

	rootCmd.AddCommand(versionCmd)

	rootCmd.PersistentFlags().StringVar(&packFile, "pack-file", "pack.toml", "The modpack metadata file to use")
	_ = viper.BindPFlag("pack-file", rootCmd.PersistentFlags().Lookup("pack-file"))

	// Make mods-folder an alias for meta-folder
	viper.RegisterAlias("mods-folder", "meta-folder")
	rootCmd.SetGlobalNormalizationFunc(func(f *pflag.FlagSet, name string) pflag.NormalizedName {
		if name == "mods-folder" {
			return "meta-folder"
		}
		return pflag.NormalizedName(name)
	})

	var metaFolder string
	rootCmd.PersistentFlags().StringVar(&metaFolder, "meta-folder", "", "The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)")
	_ = viper.BindPFlag("meta-folder", rootCmd.PersistentFlags().Lookup("meta-folder"))

	var metaFolderBase string
	rootCmd.PersistentFlags().StringVar(&metaFolderBase, "meta-folder-base", ".", "The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour)")
	_ = viper.BindPFlag("meta-folder-base", rootCmd.PersistentFlags().Lookup("meta-folder-base"))

	defaultCacheDir, err := core.GetPackwandCache()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	rootCmd.PersistentFlags().String("cache", defaultCacheDir, "The directory where packwiz will cache downloaded mods")
	_ = viper.BindPFlag("cache.directory", rootCmd.PersistentFlags().Lookup("cache"))

	file, err := core.GetPackwandLocalStore()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	file = filepath.Join(file, ".packwand.toml")
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "The config file to use (default \""+file+"\")")

	var nonInteractive bool
	rootCmd.PersistentFlags().BoolVarP(&nonInteractive, "yes", "y", false, "Accept all prompts with the default or \"yes\" option (non-interactive mode) - may pick unwanted options in search results")
	_ = viper.BindPFlag("non-interactive", rootCmd.PersistentFlags().Lookup("yes"))

	var noRefresh bool
	rootCmd.PersistentFlags().BoolVar(&noRefresh, "no-refresh", false,
		"Skip index and pack.toml refresh after modifications (use 'packwand refresh' to finalize batch operations)")
	_ = viper.BindPFlag("no-refresh", rootCmd.PersistentFlags().Lookup("no-refresh"))
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		dir, err := core.GetPackwandLocalStore()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		viper.AddConfigPath(dir)
		viper.SetConfigName(".packwand")
	}

	// Read in environment variables that match
	viper.SetEnvPrefix("packwand")
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
	viper.AutomaticEnv()

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		fmt.Println("Using config file:", viper.ConfigFileUsed())
	}
}

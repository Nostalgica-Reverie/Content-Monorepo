require 'json'

class Project
  attr_accessor :id, :name, :type, :version, :mc_versions, :modrinth_id, :curseforge_id, :author, :license

  def initialize
    @mc_versions = []
    yield(self) if block_given?
  end

  def to_h
    {
      id: @id,
      name: @name,
      type: @type,
      version: @version,
      mc_versions: @mc_versions,
      modrinth_id: @modrinth_id,
      curseforge_id: @curseforge_id,
      author: @author,
      license: @license
    }
  end
end

rc_plus = Project.new do |p|
  p.id            = "rc-plus"
  p.name          = "Re-Console Plus"
  p.type          = "modpack" # choices: modpack, datapack, resourcepack
  p.version       = "26.04.1"
  
  # ordered by publish priority
  p.mc_versions   = [
    "1.21.11-mr", 
    "1.21.11-cf", 
    "1.21.10-mr",
    "1.21.10-cf"
  ]

  p.modrinth_id   = "legacy-minecraft"
  p.curseforge_id = "re-console"
  p.author        = "omo50, Cjnator38, Technocality"
  p.license       = "GPL-3.0"
end

puts JSON.pretty_generate(rc_plus.to_h)

//
//  Copyright 2026 The InfiniFlow Authors. All Rights Reserved.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//

package cli

import (
	"fmt"
	"strings"

	"github.com/alecthomas/kingpin/v2"
)

type ListFilesArguments struct {
	paths     *[]string
	version   *bool
	all       *bool
	bytes     *bool
	mdate     *bool
	owner     *bool
	nogroup   *bool
	perms     *bool
	long      *bool
	dirs      *bool
	files     *bool
	links     *bool
	linkRel   *bool
	sortSize  *bool
	sortTime  *bool
	sortKind  *bool
	backwards *bool
	stats     *bool
	icons     *bool
	nerdfont  *bool
	recurse   *bool
	find      *string
	light     *bool
}

func (l *ListFilesArguments) PrintAll() {
	fmt.Println("paths:", strings.Join(*l.paths, " "))
	fmt.Println("version:", *l.version)
	fmt.Println("all:", *l.all)
	fmt.Println("bytes:", *l.bytes)
	fmt.Println("mdate:", *l.mdate)
	fmt.Println("owner:", *l.owner)
	fmt.Println("nogroup:", *l.nogroup)
	fmt.Println("perms:", *l.perms)
	fmt.Println("long:", *l.long)
	fmt.Println("dirs:", *l.dirs)
	fmt.Println("files:", *l.files)
	fmt.Println("links:", *l.links)
	fmt.Println("linkRel:", *l.linkRel)
	fmt.Println("sortSize:", *l.sortSize)
	fmt.Println("sortTime:", *l.sortTime)
	fmt.Println("sortKind:", *l.sortKind)
	fmt.Println("backwards:", *l.backwards)
	fmt.Println("stats:", *l.stats)
	fmt.Println("icons:", *l.icons)
	fmt.Println("nerdfont:", *l.nerdfont)
	fmt.Println("recurse:", *l.recurse)
	fmt.Println("find:", *l.find)
	fmt.Println("light:", *l.light)
}

func ParseListFileCommand(args []string) *ListFilesArguments {
	app := kingpin.New("ls", "List files and directories")
	arguments := &ListFilesArguments{}
	arguments.paths = app.Arg("paths", "the files(s) and/or folder(s) to display").Default(".").Strings()
	arguments.version = app.Flag("version", "print version and exit").Short('v').Bool()
	arguments.all = app.Flag("all", "show hidden files").Short('a').Bool()
	arguments.bytes = app.Flag("bytes", "include size").Short('b').Bool()
	arguments.mdate = app.Flag("mdate", "include modification date").Short('m').Bool()
	arguments.owner = app.Flag("owner", "include owner and group").Short('o').Bool()
	arguments.nogroup = app.Flag("nogroup", "hide group").Short('N').Bool()
	arguments.perms = app.Flag("perms", "include permissions for owner, group, and other").Short('p').Bool()
	arguments.long = app.Flag("long", "include size, date, owner, and permissions").Short('l').Bool()
	arguments.dirs = app.Flag("dirs", "only show directories").Short('d').Bool()
	arguments.files = app.Flag("files", "only show files").Short('f').Bool()
	arguments.links = app.Flag("links", "show paths for symlinks").Short('L').Bool()
	arguments.linkRel = app.Flag("link-rel", "show symlinks as relative paths if shorter than absolute path").Short('R').Bool()
	arguments.sortSize = app.Flag("size", "sort items by size").Short('s').Bool()
	arguments.sortTime = app.Flag("time", "sort items by time").Short('t').Bool()
	arguments.sortKind = app.Flag("kind", "sort items by extension").Short('k').Bool()
	arguments.backwards = app.Flag("backwards", "reverse the sort order of --size, --time, or --kind").Short('B').Bool()
	arguments.stats = app.Flag("stats", "show statistics").Short('S').Bool()
	arguments.icons = app.Flag("icons", "show folder icon before dirs").Short('i').Bool()
	arguments.nerdfont = app.Flag("nerd-font", "show nerd font glyphs before file names").Short('n').Bool()
	arguments.recurse = app.Flag("recurse", "traverse all dirs recursively").Short('r').Bool()
	arguments.find = app.Flag("find", "filter items with a regexp").Short('F').String()
	arguments.light = app.Flag("light", "output colors for light-bachground themes").Short('I').Bool()
	kingpin.MustParse(app.Parse(args))
	return arguments
}

func (c *CLI) SingleListFilesCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	argv := strings.Split(cmd.Params["command"].(string), " ")
	arguments := ParseListFileCommand(argv)
	arguments.PrintAll()
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = "List files not implemented"
	return &result, nil
}

func (c *CLI) SingleCatFileCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = "Cat file not implemented"
	return &result, nil
}

func (c *CLI) SingleSearchCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = "Search not implemented"
	return &result, nil
}

func (c *CLI) SingleFindCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = "Find not implemented"
	return &result, nil
}

func (c *CLI) SingleUseWorkspaceCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 0
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = fmt.Sprintf("Use workspace %s", cmd.Params["workspace"])
	return &result, nil
}

func (c *CLI) SingleNewWorkspaceCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = fmt.Sprintf("New workspace %s", cmd.Params["new_workspace"])
	return &result, nil
}

func (c *CLI) SingleDeleteWorkspaceCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = fmt.Sprintf("Delete workspace %s", cmd.Params["workspace"])
	return &result, nil
}

func (c *CLI) SingleSaveMessageCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = fmt.Sprintf("Attempt to save message: %s", cmd.Params["message"])
	return &result, nil
}

func (c *CLI) SingleShowCurrentWorkspaceCommand(commandCount int, cmd *Command) (ResponseIf, error) {
	result := SimpleResponse{}
	result.Code = 1
	result.SetOutputFormat(c.Config.OutputFormat)
	result.Message = "Show current workspace not implemented"
	return &result, nil
}

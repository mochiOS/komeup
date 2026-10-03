#!/usr/bin/env perl

use strict;
use warnings;
use Cwd qw(abs_path);
use Digest::SHA ();
use File::Basename qw(basename);
use File::Copy qw(copy);
use File::Path qw(make_path);
use File::Spec;
use File::Temp qw(tempdir);
use FindBin qw($Bin);
use Getopt::Long qw(GetOptions);

my $root = abs_path(File::Spec->catdir($Bin, '..'));
my ($sdk_version, $target, $arch);
GetOptions(
    'sdk-version=s' => \$sdk_version,
    'target=s' => \$target,
    'arch=s' => \$arch,
) or die usage();
defined $sdk_version or die usage();
$sdk_version =~ /\A[A-Za-z0-9][A-Za-z0-9._+-]*\z/ or die "invalid SDK version\n";

if (!defined $arch) {
    if (defined $target) {
        ($arch) = split /-/, $target, 2;
    } else {
        open my $rustc, '-|', 'rustc', '-vV' or die "failed to start rustc: $!\n";
        while (my $line = <$rustc>) { $arch = $1 if $line =~ /^host:\s+([^-\s]+)/; }
        close $rustc or die "rustc failed\n";
    }
}
defined $arch or die "architecture was not detected\n";

my @build = ('cargo', 'build', '--release', '--locked');
push @build, '--target', $target if defined $target;
run(@build);
my $binary = defined $target
    ? File::Spec->catfile($root, 'target', $target, 'release', 'komeup')
    : File::Spec->catfile($root, 'target', 'release', 'komeup');
-f $binary or die "release binary was not found: $binary\n";

my $output = File::Spec->catdir($root, 'target', 'release');
make_path($output);
my $stage = tempdir('komeup-release-XXXXXX', TMPDIR => 1, CLEANUP => 1);
copy($binary, File::Spec->catfile($stage, 'komeup')) or die "failed to stage komeup: $!\n";
chmod 0755, File::Spec->catfile($stage, 'komeup');
my $name = "$arch-komeup-$sdk_version";
my $tar = File::Spec->catfile($stage, "$name.tar");
my $archive = File::Spec->catfile($output, "$name.tar.zst");
run('tar', '--sort=name', '--mtime=@' . source_date_epoch(), '--owner=0', '--group=0',
    '--numeric-owner', '-C', $stage, '-cf', $tar, 'komeup');
run('zstd', '-q', '-19', '-f', $tar, '-o', $archive);
open my $input, '<', $archive or die "failed to read archive: $!\n";
binmode $input;
my $digest = Digest::SHA->new(256)->addfile($input)->hexdigest;
close $input;
open my $sums, '>', File::Spec->catfile($output, 'SHA256SUMS') or die "failed to write checksums: $!\n";
print {$sums} "$digest  " . basename($archive) . "\n";
close $sums;
print "$archive\n";

sub usage { "usage: scripts/release.pl --sdk-version <version> [--target <target>] [--arch <arch>]\n" }
sub source_date_epoch {
    open my $git, '-|', 'git', '-C', $root, 'log', '-1', '--format=%ct' or die "failed to start git: $!\n";
    my $epoch = <$git>; close $git; chomp $epoch; return $epoch;
}
sub run {
    my (@command) = @_; print '+ ' . join(' ', @command) . "\n"; system @command;
    ($? >> 8) == 0 or die "$command[0] failed\n";
}

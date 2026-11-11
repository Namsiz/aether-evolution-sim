#!/usr/bin/perl
use strict;
use warnings;
use List::Util qw(sum min max);

# ------------------------------------------------------------
# 世界参数。改这里就是改这个世界的"物理常数"。
# ------------------------------------------------------------
my %P = (
    K            => 4,      # 元素种类
    N0           => 500,    # 初始存在物数量
    W            => 600,    # 世界边长
    CLUSTERS     => 6,      # 初始物质团块数
    CLUSTER_SIGMA => 78,    # 团块半径
    FUSE_E       => 2.5,    # 聚变所需的总能量
    FUSE_LOSS    => 0.55,   # 聚变释放掉的能量（不是消失，是离开局部）
    FISSION_E    => 10.0,   # 裂变阈值（只有真正的大家伙才会裂开）
    FISSION_LOSS => 0.35,
    RADIATE_E    => 9.0,    # 开始辐射的阈值
    RADIATE_COST => 3.0,    # 每次辐射消耗
    BREAK_R      => 42.0,   # 键被拉到多长就断
    BOND_R       => 25.0,   # 成键距离
    BOND_L       => 11.0,   # 键的静止长度
    SPRING       => 0.03,   # 键的弹性（同时也是近距排斥）
    GRAVITY      => 0.6,    # 局部引力强度
    DAMPING      => 0.985,  # 速度阻尼
    MAX_SPEED    => 2.5,    # 速度上限
    MAX_DEG      => 7,      # 每个存在物最多几个键
    DIFFUSE      => 0.035,  # 沿键的能量传导率
    SEED_N       => 8,      # 每步接受背景能量的存在物数
    SEED_E       => 1.2,
    P_FUSE       => 0.01,
    P_FISSION    => 0.05,
    P_RADIATE    => 0.3,
    P_BOND       => 0.6,
    P_BREAK      => 0.1,
    P_DECAY      => 0.12,
    BOND_TRIES   => 400,    # 每步随机试探的成键对数
    BOND_QUOTA   => 18,     # 每步成键的独立配额
    MAX_EVENTS   => 20,     # 每步最多发生几个化学事件
);

# ------------------------------------------------------------
# 世界：节点表 + 边表 + 事件图
# 事件图是历史本身。节点的 last 指向"最近一次改变它的事件"，
# 新事件的 layer = 所有前驱事件的 layer 最大值 + 1。
# 于是世界的时间不是一条线，而是一张有层级的偏序图。
# ------------------------------------------------------------
sub new_world {
    return {
        nodes   => {},
        edges   => {},
        adj     => {},
        events  => [],
        next_id => 0,
        step    => 0,
    };
}

sub add_node {
    my ($w, $t, $e, $x, $y, $born, $last, $vx, $vy) = @_;
    my $id = $w->{next_id}++;
    $x = 20 if $x < 20; $x = $P{W} - 20 if $x > $P{W} - 20;
    $y = 20 if $y < 20; $y = $P{W} - 20 if $y > $P{W} - 20;
    $w->{nodes}{$id} = {
        t => $t, e => $e, x => $x, y => $y,
        vx => ($vx // 0), vy => ($vy // 0),
        born => $born, last => $last,
    };
    $w->{adj}{$id} = {};
    return $id;
}

sub add_edge {
    my ($w, $a, $b) = @_;
    return if !defined $a || !defined $b || $a == $b;
    return unless $w->{nodes}{$a} && $w->{nodes}{$b};
    my ($lo, $hi) = $a < $b ? ($a, $b) : ($b, $a);
    return if $w->{edges}{"$lo-$hi"};
    $w->{edges}{"$lo-$hi"} = 1;
    $w->{adj}{$lo}{$hi} = 1;
    $w->{adj}{$hi}{$lo} = 1;
}

sub del_edge {
    my ($w, $a, $b) = @_;
    my ($lo, $hi) = $a < $b ? ($a, $b) : ($b, $a);
    delete $w->{edges}{"$lo-$hi"};
    delete $w->{adj}{$lo}{$hi} if $w->{adj}{$lo};
    delete $w->{adj}{$hi}{$lo} if $w->{adj}{$hi};
}

sub remove_node {
    my ($w, $id) = @_;
    return unless $w->{nodes}{$id};
    for my $nb (keys %{ $w->{adj}{$id} }) {
        del_edge($w, $id, $nb);
    }
    delete $w->{adj}{$id};
    delete $w->{nodes}{$id};
}

sub dist {
    my ($a, $b) = @_;
    my $dx = $a->{x} - $b->{x};
    my $dy = $a->{y} - $b->{y};
    return sqrt($dx * $dx + $dy * $dy);
}

sub emit_event {
    my ($w, $rule, $deps, $detail) = @_;
    my $id = scalar @{ $w->{events} };
    my $layer = 1;
    for my $d (@$deps) {
        next unless defined $d;
        my $ev = $w->{events}[$d];
        next unless $ev;
        $layer = $ev->{layer} + 1 if $ev->{layer} + 1 > $layer;
    }
    push @{ $w->{events} }, {
        id => $id, layer => $layer, deps => $deps,
        rule => $rule, step => $w->{step}, detail => $detail,
    };
    return $id;
}

# ------------------------------------------------------------
# 规则：只读局部，绝不改全局。
# 每条规则返回一批"匹配"，由调度器决定谁真的发生。
# ------------------------------------------------------------
sub match_fuse {
    my ($w) = @_;
    my @m;
    for my $key (keys %{ $w->{edges} }) {
        my ($a, $b) = split /-/, $key;
        my $na = $w->{nodes}{$a};
        my $nb = $w->{nodes}{$b};
        next unless $na && $nb;
        next unless $na->{e} + $nb->{e} >= $P{FUSE_E};
        push @m, { rule => 'fuse', nodes => [$a, $b], prob => $P{P_FUSE} };
    }
    return @m;
}

sub match_fission {
    my ($w) = @_;
    my @m;
    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        next unless $n->{e} >= $P{FISSION_E};
        push @m, { rule => 'fission', nodes => [$id], prob => $P{P_FISSION} };
    }
    return @m;
}

sub build_grid {
    my ($w, $cell) = @_;
    my %g;
    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        push @{ $g{ int($n->{x} / $cell) . ',' . int($n->{y} / $cell) } }, $id;
    }
    return \%g;
}

sub match_bond {
    my ($w) = @_;
    my @m;
    my @ids = keys %{ $w->{nodes} };
    return @m if @ids < 2;
    my $grid = build_grid($w, $P{BOND_R});
    my %taken;
    for (1 .. $P{BOND_TRIES}) {
        my $a = $ids[ int(rand(@ids)) ];
        next if $taken{$a};
        my $na = $w->{nodes}{$a};
        next unless $na;
        next if scalar(keys %{ $w->{adj}{$a} }) >= $P{MAX_DEG};

        my $cx = int($na->{x} / $P{BOND_R});
        my $cy = int($na->{y} / $P{BOND_R});
        my ($best, $bd);
        for my $dx (-1 .. 1) {
            for my $dy (-1 .. 1) {
                for my $b (@{ $grid->{ ($cx + $dx) . ',' . ($cy + $dy) } || [] }) {
                    next if $b == $a || $w->{adj}{$a}{$b};
                    my $nb = $w->{nodes}{$b};
                    next unless $nb;
                    next if scalar(keys %{ $w->{adj}{$b} }) >= $P{MAX_DEG};
                    # 只允许互补的元素成键。这条规则会产生"化学"。
                    next unless ($na->{t} + $nb->{t}) % $P{K} == 1;
                    my $d = dist($na, $nb);
                    next unless $d <= $P{BOND_R};
                    if (!defined $bd || $d < $bd) { $bd = $d; $best = $b; }
                }
            }
        }
        next unless defined $best;
        $taken{$a} = 1;
        $taken{$best} = 1;
        push @m, { rule => 'bond', nodes => [$a, $best], prob => $P{P_BOND} };
    }
    return @m;
}

sub match_radiate {
    my ($w) = @_;
    my @m;
    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        next unless $n->{e} >= $P{RADIATE_E};
        push @m, { rule => 'radiate', nodes => [$id], prob => $P{P_RADIATE} };
    }
    return @m;
}

sub match_break {
    my ($w) = @_;
    my @m;
    for my $key (keys %{ $w->{edges} }) {
        my ($a, $b) = split /-/, $key;
        my $na = $w->{nodes}{$a};
        my $nb = $w->{nodes}{$b};
        next unless $na && $nb;
        next unless dist($na, $nb) > $P{BREAK_R};
        push @m, { rule => 'break', nodes => [$a, $b], prob => $P{P_BREAK} };
    }
    return @m;
}

sub match_decay {
    my ($w) = @_;
    my @m;
    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        next unless $n->{e} < 0.12;
        next unless scalar(keys %{ $w->{adj}{$id} }) == 0;
        push @m, { rule => 'decay', nodes => [$id], prob => $P{P_DECAY} };
    }
    return @m;
}

# ------------------------------------------------------------
# 事件对世界的改写
# ------------------------------------------------------------
sub apply_fuse {
    my ($w, $m) = @_;
    my ($a, $b) = @{ $m->{nodes} };
    my $na = $w->{nodes}{$a};
    my $nb = $w->{nodes}{$b};
    return unless $na && $nb;

    my $eid   = scalar @{ $w->{events} };
    my $deps  = [ $na->{last}, $nb->{last} ];
    my $layer = 1;
    for my $d (@$deps) {
        my $ev = $w->{events}[$d];
        $layer = $ev->{layer} + 1 if $ev && $ev->{layer} + 1 > $layer;
    }

    my $ne = $na->{e} + $nb->{e} - $P{FUSE_LOSS};
    my $nt = ($na->{t} + $nb->{t} + 1) % $P{K};
    my $nx = ($na->{x} + $nb->{x}) / 2;
    my $ny = ($na->{y} + $nb->{y}) / 2;

    my $nvx = ($na->{vx} + $nb->{vx}) / 2;
    my $nvy = ($na->{vy} + $nb->{vy}) / 2;

    my %keep = map { $_ => 1 } (keys %{ $w->{adj}{$a} }, keys %{ $w->{adj}{$b} });
    delete $keep{$a};
    delete $keep{$b};

    remove_node($w, $a);
    remove_node($w, $b);
    my $id = add_node($w, $nt, $ne, $nx, $ny, $eid, $eid, $nvx, $nvy);
    for my $k (keys %keep) {
        add_edge($w, $id, $k);
    }

    push @{ $w->{events} }, {
        id => $eid, layer => $layer, deps => $deps,
        rule => 'fuse', step => $w->{step}, detail => "$a + $b -> $id",
    };
}

sub apply_fission {
    my ($w, $m) = @_;
    my $id = $m->{nodes}[0];
    my $n  = $w->{nodes}{$id};
    return unless $n;

    my $eid   = scalar @{ $w->{events} };
    my $prev  = $w->{events}[ $n->{last} ];
    my $layer = $prev ? $prev->{layer} + 1 : 1;

    my $e1 = $n->{e} / 2 - $P{FISSION_LOSS};
    my $e2 = $n->{e} / 2 - $P{FISSION_LOSS};
    my $t1 = ($n->{t} + 1) % $P{K};
    my $t2 = ($n->{t} + 2) % $P{K};
    my $x1 = $n->{x} + (rand() - 0.5) * 46;
    my $y1 = $n->{y} + (rand() - 0.5) * 46;
    my $x2 = $n->{x} + (rand() - 0.5) * 46;
    my $y2 = $n->{y} + (rand() - 0.5) * 46;
    my @nbr = keys %{ $w->{adj}{$id} };
    my $pvx = $n->{vx};
    my $pvy = $n->{vy};

    remove_node($w, $id);
    my $id1 = add_node($w, $t1, $e1, $x1, $y1, $eid, $eid, $pvx + (rand() - 0.5), $pvy + (rand() - 0.5));
    my $id2 = add_node($w, $t2, $e2, $x2, $y2, $eid, $eid, $pvx + (rand() - 0.5), $pvy + (rand() - 0.5));
    my $i = 0;
    for my $k (@nbr) {
        $i++;
        add_edge($w, ($i % 2 ? $id1 : $id2), $k);
    }

    push @{ $w->{events} }, {
        id => $eid, layer => $layer, deps => [ $n->{last} ],
        rule => 'fission', step => $w->{step}, detail => "$id -> $id1 + $id2",
    };
}

sub apply_bond {
    my ($w, $m) = @_;
    my ($a, $b) = @{ $m->{nodes} };
    my $na = $w->{nodes}{$a};
    my $nb = $w->{nodes}{$b};
    return unless $na && $nb;

    my $eid = emit_event($w, 'bond', [ $na->{last}, $nb->{last} ], "$a - $b");
    add_edge($w, $a, $b);
    $w->{nodes}{$a}{last} = $eid;
    $w->{nodes}{$b}{last} = $eid;
}

sub apply_radiate {
    my ($w, $m) = @_;
    my $id = $m->{nodes}[0];
    my $n  = $w->{nodes}{$id};
    return unless $n;

    my $eid = emit_event($w, 'radiate', [ $n->{last} ], "* $id");
    $n->{e} -= $P{RADIATE_COST};
    $n->{last} = $eid;

    my $ang = rand() * 6.28318;
    my $r   = 24 + rand(26);
    my $x   = $n->{x} + cos($ang) * $r;
    my $y   = $n->{y} + sin($ang) * $r;
    my $t   = int(rand($P{K}));
    my $new = add_node(
        $w, $t, 1.0, $x, $y, $eid, $eid,
        $n->{vx} + cos($ang) * 1.1, $n->{vy} + sin($ang) * 1.1
    );

    # 抛出的物质会立刻和母体建立一条长程键，形成"行星轨道"式的连接
    add_edge($w, $new, $id) if rand() < 0.35;
}

sub apply_break {
    my ($w, $m) = @_;
    my ($a, $b) = @{ $m->{nodes} };
    return unless $w->{nodes}{$a} && $w->{nodes}{$b};
    my $eid = emit_event($w, 'break', [ $w->{nodes}{$a}{last}, $w->{nodes}{$b}{last} ], "$a / $b");
    del_edge($w, $a, $b);
    $w->{nodes}{$a}{last} = $eid;
    $w->{nodes}{$b}{last} = $eid;
}

sub apply_decay {
    my ($w, $m) = @_;
    my $id = $m->{nodes}[0];
    my $n  = $w->{nodes}{$id};
    return unless $n;

    emit_event($w, 'decay', [ $n->{last} ], "~ $id");
    remove_node($w, $id);
}

# ------------------------------------------------------------
# 连续层：能量沿键传导 + 背景注入。
# 拓扑变化记入事件图，能量流动不记。这是有意的取舍。
# ------------------------------------------------------------
sub physics {
    my ($w) = @_;
    my (%fx, %fy);

    # 键：传能 + 弹簧
    for my $key (keys %{ $w->{edges} }) {
        my ($a, $b) = split /-/, $key;
        my $na = $w->{nodes}{$a};
        my $nb = $w->{nodes}{$b};
        next unless $na && $nb;

        my $de = ($na->{e} - $nb->{e}) * $P{DIFFUSE};
        $na->{e} -= $de;
        $nb->{e} += $de;

        my $dx = $nb->{x} - $na->{x};
        my $dy = $nb->{y} - $na->{y};
        my $len = sqrt($dx * $dx + $dy * $dy) + 1e-6;
        my $f = ($len - $P{BOND_L}) * $P{SPRING};
        my $ux = $dx / $len;
        my $uy = $dy / $len;
        $fx{$a} += $f * $ux;  $fy{$a} += $f * $uy;
        $fx{$b} -= $f * $ux;  $fy{$b} -= $f * $uy;
    }

    # 引力：只算近邻，质量就是能量
    my $grid = build_grid($w, 110);
    my (%gx, %gy);
    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        my $cx = int($n->{x} / 110);
        my $cy = int($n->{y} / 110);
        my ($sumx, $sumy) = (0, 0);
        for my $dx (-1 .. 1) {
            for my $dy (-1 .. 1) {
                for my $j (@{ $grid->{ ($cx + $dx) . ',' . ($cy + $dy) } || [] }) {
                    next if $j == $id;
                    my $m = $w->{nodes}{$j};
                    my $ddx = $m->{x} - $n->{x};
                    my $ddy = $m->{y} - $n->{y};
                    my $d2 = $ddx * $ddx + $ddy * $ddy + 120;
                    my $d = sqrt($d2);
                    my $f = $P{GRAVITY} * $m->{e} / $d2;
                    $sumx += $f * $ddx / $d;
                    $sumy += $f * $ddy / $d;
                }
            }
        }
        $gx{$id} = $sumx;
        $gy{$id} = $sumy;
    }

    for my $id (keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        my $ax = ($fx{$id} || 0) + ($gx{$id} || 0);
        my $ay = ($fy{$id} || 0) + ($gy{$id} || 0);

        $n->{vx} = ($n->{vx} + $ax) * $P{DAMPING};
        $n->{vy} = ($n->{vy} + $ay) * $P{DAMPING};

        my $sp = sqrt($n->{vx} * $n->{vx} + $n->{vy} * $n->{vy});
        if ($sp > $P{MAX_SPEED}) {
            $n->{vx} *= $P{MAX_SPEED} / $sp;
            $n->{vy} *= $P{MAX_SPEED} / $sp;
        }

        $n->{x} += $n->{vx};
        $n->{y} += $n->{vy};

        if ($n->{x} < 20)              { $n->{x} = 20;             $n->{vx} =  abs($n->{vx}) * 0.6; }
        if ($n->{x} > $P{W} - 20)      { $n->{x} = $P{W} - 20;     $n->{vx} = -abs($n->{vx}) * 0.6; }
        if ($n->{y} < 20)              { $n->{y} = 20;             $n->{vy} =  abs($n->{vy}) * 0.6; }
        if ($n->{y} > $P{W} - 20)      { $n->{y} = $P{W} - 20;     $n->{vy} = -abs($n->{vy}) * 0.6; }
    }

    my @ids = keys %{ $w->{nodes} };
    return unless @ids;
    for (1 .. $P{SEED_N}) {
        my $id = $ids[ int(rand(@ids)) ];
        $w->{nodes}{$id}{e} += $P{SEED_E};
    }
}

# ------------------------------------------------------------
# 观察接口：你只能看见你查询的那一箱世界。
# ------------------------------------------------------------
sub observe {
    my ($w, $x0, $y0, $x1, $y1) = @_;
    my @ids = grep {
        my $n = $w->{nodes}{$_};
        $n->{x} >= $x0 && $n->{x} <= $x1 && $n->{y} >= $y0 && $n->{y} <= $y1;
    } keys %{ $w->{nodes} };
    my $energy = 0;
    $energy += $w->{nodes}{$_}{e} for @ids;
    return { ids => \@ids, n => scalar @ids, energy => $energy };
}

# ------------------------------------------------------------
# 结构指标：用来看世界到底长出了什么
# ------------------------------------------------------------
sub components {
    my ($w) = @_;
    my %seen;
    my $count = 0;
    my $largest = 0;
    for my $start (keys %{ $w->{nodes} }) {
        next if $seen{$start};
        $count++;
        my @stack = ($start);
        $seen{$start} = 1;
        my $size = 0;
        while (@stack) {
            my $id = pop @stack;
            $size++;
            for my $nb (keys %{ $w->{adj}{$id} || {} }) {
                next if $seen{$nb};
                $seen{$nb} = 1;
                push @stack, $nb;
            }
        }
        $largest = $size if $size > $largest;
    }
    return ($count, $largest);
}

sub report {
    my ($w, $step) = @_;
    my @ids = keys %{ $w->{nodes} };
    my $n = scalar @ids;
    my $edges = scalar keys %{ $w->{edges} };
    my $energy = 0;
    my $maxe = 0;
    my %t;
    for my $id (@ids) {
        my $node = $w->{nodes}{$id};
        $energy += $node->{e};
        $maxe = $node->{e} if $node->{e} > $maxe;
        $t{ $node->{t} }++;
    }
    my ($comp, $largest) = components($w);
    my $deg = $n ? 2 * $edges / $n : 0;
    my $cycles = $n ? $edges - $n + $comp : 0;
    my $types = join(" ", map { "$_:$t{$_}" } sort keys %t);
    my $maxlayer = 0;
    for my $ev (@{ $w->{events} }) {
        $maxlayer = $ev->{layer} if $ev->{layer} > $maxlayer;
    }
    printf "step %5d | 存在物 %4d | 键 %5d | 平均度 %4.2f | 环 %4d | 分量 %3d(最大 %3d) | 能量 %7.1f(峰 %5.2f) | 类型 %s | 事件 %6d | 因果深度 %5d\n",
        $step, $n, $edges, $deg, $cycles, $comp, $largest, $energy, $maxe, $types,
        scalar @{ $w->{events} }, $maxlayer;
}

# ------------------------------------------------------------
# 渲染：一张 SVG 快照。颜色是元素，大小是能量。
# ------------------------------------------------------------
sub type_rgb {
    my ($t) = @_;
    my @base = ([106, 168, 255], [95, 224, 200], [255, 180, 94], [201, 139, 255]);
    return @{ $base[ $t % 4 ] };
}

sub type_color {
    my ($t, $e) = @_;
    my @b = type_rgb($t);
    my $k = 0.6 + 0.4 * min(1, $e / 8);
    return sprintf("#%02x%02x%02x", map { int(min(255, $_ * $k)) } @b);
}

sub render_svg {
    my ($w, $path) = @_;
    my $W = $P{W};
    my @out;
    push @out, qq{<svg xmlns="http://www.w3.org/2000/svg" width="$W" height="$W" viewBox="0 0 $W $W">};
    push @out, qq{<rect width="$W" height="$W" fill="#04060c"/>};

    # 大质量存在物的光晕
    push @out, '<defs>';
    for my $t (0 .. $P{K} - 1) {
        my ($r, $g, $b) = type_rgb($t);
        push @out, qq{<radialGradient id="glow$t">};
        push @out, qq{<stop offset="0%" stop-color="rgb($r,$g,$b)" stop-opacity="0.40"/>};
        push @out, qq{<stop offset="45%" stop-color="rgb($r,$g,$b)" stop-opacity="0.12"/>};
        push @out, qq{<stop offset="100%" stop-color="rgb($r,$g,$b)" stop-opacity="0"/>};
        push @out, '</radialGradient>';
    }
    push @out, '</defs>';

    my @edges;
    for my $key (keys %{ $w->{edges} }) {
        my ($a, $b) = split /-/, $key;
        my $na = $w->{nodes}{$a};
        my $nb = $w->{nodes}{$b};
        next unless $na && $nb;
        push @edges, sprintf(
            qq{<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="#3a5c96" stroke-width="0.6" opacity="0.40"/>},
            $na->{x}, $na->{y}, $nb->{x}, $nb->{y}
        );
    }
    push @out, @edges;

    for my $id (sort { $w->{nodes}{$a}{e} <=> $w->{nodes}{$b}{e} } keys %{ $w->{nodes} }) {
        my $n = $w->{nodes}{$id};
        my $r = 1.2 + ($n->{e} ** 0.62) * 2.4;

        if ($n->{e} > 5) {
            my $gr = $r * (1.5 + $n->{e} * 0.07);
            push @out, sprintf(
                qq{<circle cx="%.1f" cy="%.1f" r="%.1f" fill="url(#glow%d)"/>},
                $n->{x}, $n->{y}, $gr, $n->{t}
            );
        }

        my $c = type_color($n->{t}, $n->{e});
        my $o = 0.55 + 0.4 * min(1, $n->{e} / 8);
        push @out, sprintf(
            qq{<circle cx="%.1f" cy="%.1f" r="%.1f" fill="%s" opacity="%.2f"/>},
            $n->{x}, $n->{y}, $r, $c, $o
        );
    }
    push @out, '</svg>';

    open my $fh, '>', $path or die "cannot write $path: $!";
    print $fh join("\n", @out), "\n";
    close $fh;
}

# ------------------------------------------------------------
# 创世
# ------------------------------------------------------------
sub gauss {
    my $s = 0;
    $s += rand() for 1 .. 6;
    return ($s - 3) * 1.414;
}

sub big_bang {
    my $w = new_world();
    my @centers;
    for (1 .. $P{CLUSTERS}) {
        push @centers, [ 70 + rand($P{W} - 140), 70 + rand($P{W} - 140) ];
    }
    for my $i (1 .. $P{N0}) {
        my $c = $centers[ int(rand(@centers)) ];
        my $x = $c->[0] + gauss() * $P{CLUSTER_SIGMA};
        my $y = $c->[1] + gauss() * $P{CLUSTER_SIGMA};
        my $t = int(rand($P{K}));
        my $e = 0.6 + rand(1.2);
        my $dx = $x - $c->[0];
        my $dy = $y - $c->[1];
        my $rr = sqrt($dx * $dx + $dy * $dy) + 1e-6;
        my $vt = 0.9 + rand(0.5);
        my $vx = -$dy / $rr * $vt + (rand() - 0.5) * 0.3;
        my $vy =  $dx / $rr * $vt + (rand() - 0.5) * 0.3;
        add_node($w, $t, $e, $x, $y, 0, 0, $vx, $vy);
    }
    my @ids = keys %{ $w->{nodes} };
    for my $i (0 .. $#ids - 1) {
        for my $j ($i + 1 .. $#ids) {
            my $na = $w->{nodes}{ $ids[$i] };
            my $nb = $w->{nodes}{ $ids[$j] };
            add_edge($w, $ids[$i], $ids[$j]) if dist($na, $nb) < 18;
        }
    }
    return $w;
}

# ------------------------------------------------------------
# 演化
# ------------------------------------------------------------
my $steps   = shift @ARGV // 4000;
my $outfile = shift @ARGV // 'state.svg';

my $w = big_bang();
report($w, 0);

for my $step (1 .. $steps) {
    $w->{step} = $step;

    my %used;

    # 成键是几何过程，不和化学事件抢预算。
    my $bond_budget = $P{BOND_QUOTA};
    for my $m (match_bond($w)) {
        last if $bond_budget <= 0;
        next if $m->{prob} < 1 && rand() > $m->{prob};
        next if grep { $used{$_} } @{ $m->{nodes} };
        apply_bond($w, $m);
        $used{$_} = 1 for @{ $m->{nodes} };
        $bond_budget--;
    }

    my @matches = (
        match_fuse($w),
        match_fission($w),
        match_radiate($w),
        match_break($w),
        match_decay($w),
    );
    @matches = sort { rand() <=> rand() } @matches;

    my $budget = $P{MAX_EVENTS};
    for my $m (@matches) {
        last if $budget <= 0;
        next if $m->{prob} < 1 && rand() > $m->{prob};
        my $conflict = 0;
        for my $id (@{ $m->{nodes} }) {
            if ($used{$id}) { $conflict = 1; last; }
        }
        next if $conflict;

        if    ($m->{rule} eq 'fuse')    { apply_fuse($w, $m); }
        elsif ($m->{rule} eq 'fission') { apply_fission($w, $m); }
        elsif ($m->{rule} eq 'radiate') { apply_radiate($w, $m); }
        elsif ($m->{rule} eq 'bond')    { apply_bond($w, $m); }
        elsif ($m->{rule} eq 'break')   { apply_break($w, $m); }
        elsif ($m->{rule} eq 'decay')   { apply_decay($w, $m); }

        $used{$_} = 1 for @{ $m->{nodes} };
        $budget--;
    }

    physics($w);
    report($w, $step) if $step % 250 == 0;
}

render_svg($w, $outfile);
report($w, $steps);

# 观察者只能看见自己查询的那一箱世界
for my $win ([100, 100, 300, 300], [300, 100, 500, 300], [200, 250, 400, 450]) {
    my $o = observe($w, @$win);
    printf "观察者窗口 (%d,%d)-(%d,%d): %d 个存在物 | 总能量 %.1f | 平均能量 %.2f\n",
        @$win, $o->{n}, $o->{energy}, ($o->{n} ? $o->{energy} / $o->{n} : 0);
}

print "snapshot -> $outfile\n";

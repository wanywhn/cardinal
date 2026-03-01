#include <QTreeWidget>
#include <QListWidgetItem>
#include <QApplication>
#include <QDesktopServices>
#include <QUrl>

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);

    QTreeWidget *treeWidget = new QTreeWidget();
    treeWidget->setColumnCount(1);
    QList<QTreeWidgetItem *> items;
    for (int i = 0; i < 10; ++i)
        items.append(new QTreeWidgetItem(static_cast<QTreeWidget *>(nullptr), QStringList(QString("item: %1").arg(i))));
    treeWidget->insertTopLevelItems(0, items);
    treeWidget->show();
    QDesktopServices::openUrl(QUrl("file:////storage/Users/currentUser/Documents/nihao.txt", QUrl::TolerantMode));
    return app.exec();
}